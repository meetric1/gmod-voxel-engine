use rglua::prelude::*;
use glam::*;
use node::*;
use col::*;

pub mod node;
pub mod col;

include!("helpers.rs");

#[lua_function]
fn collectgarbage(l : LuaState) -> i32 {
	let octree_userdata = check_octree(l, 1);
	unsafe {std::ptr::drop_in_place(octree_userdata)};
	return 0;
}

// probably will crash when given incorrect parameters
#[lua_function]
fn fill(l : LuaState) -> i32 {
	let octree = lua_checkoctree(l, 1);
	let size = (1 << octree.size) >> 1;	// offset octree position so 0,0,0 is in the middle

	// get position to fill
	let pos = luaL_checkvector(l, 2);

	// get color (cringe)
	let r = luaL_checknumber(l, 3) as u8;
	let g = luaL_checknumber(l, 4) as u8;
	let b = luaL_checknumber(l, 5) as u8;
	let a = luaL_checknumber(l, 6) as u8;
	
	// return table of invalid / filled positions
	lua_newtable(l);
	
	// casting to int rounds towards 0, we dont want that so we have to floor it
	octree.fill(vector_2_ivec3(pos), Color::new(r, g, b, a), -ivec3(size, size, size));
	return 1;
}

#[lua_function]
fn get_changed_data(l : LuaState) -> i32 {
	let octree = lua_checkoctree(l, 1);
	let size = (1 << octree.size) >> 1;
	let sort = luaL_checkinteger(l, 3);
	lua_pop(l, 1);

	octree.get_changed_data(l, -ivec3(size, size, size), sort != 0);   
	return 1;
}

#[lua_function]
fn get_data(l : LuaState) -> i32 {
	let octree = lua_checkoctree(l, 1);
	let size = (1 << octree.size) >> 1;

	octree.get_data(l, -ivec3(size, size, size));   
	return 1;
}

#[lua_function]
fn optimize(l : LuaState) -> i32 {
	let octree = lua_checkoctree(l, 1);
	let size = (1 << octree.size) >> 1;

	octree.cull_changed(ivec3(0, 0, 0));	// cull initial changed nodes
	octree.optimize_changed(l, -ivec3(size, size, size));	// optimize changed nodes
	octree.cull_merged(ivec3(0, 0, 0));	// cull optimized changed nodes
	
	return 1;
}

#[lua_function]
fn fill_sdf(l : LuaState) -> i32 {
	let octree = lua_checkoctree(l, 1);
	let size = (1 << octree.size) >> 1;

	// get position to fill
	let mut handles = Vec::<std::thread::JoinHandle<_>>::new();
	octree.fill_sdf(-ivec3(size, size, size));
	for child in octree.children.as_mut().unwrap() {
		handles.push(std::thread::spawn(|| {
			child.cull_changed(ivec3(0, 0, 0));
		}));
	}

	for handle in handles {
		handle.join().unwrap();
	}

	//octree.cull_changed(ivec3(0, 0, 0));
	return 0;
}

#[lua_function]
fn get_voxels(l : LuaState) -> i32 {
	let octree = lua_checkoctree(l, 1);
	let size = (1 << octree.size) >> 1;

	let pos = vector_2_ivec3(luaL_checkvector(l, 2));

	let node = octree.node_at_pos_down(pos, -ivec3(size, size, size));
	if !node.is_null() {
		unsafe {(*node).push_lua_data(l, pos)};
	}
	else {
		lua_pushnil(l);
	}

	return 1;
}

#[lua_function]
fn intersect(l : LuaState) -> i32 {
	let octree = lua_checkoctree(l, 1);
	let size = (1 << octree.size) >> 1;

	let ray_pos = luaL_checkvector(l, 2);
	let ray_dir = luaL_checkvector(l, 3);

	lua_pushnumber(l, octree.intersect(vector_2_vec3(ray_pos), 1.0 / vector_2_vec3(ray_dir), -ivec3(size, size, size)) as f64);
	return 1;
}

#[lua_function]
fn new(l: LuaState) -> i32 {
	let size = luaL_checknumber(l, 1) as u8;
	if size > 30 {
		luaL_error(l, cstr!("Octree size must be less than 31!"));
	}

	push_octree(l, &Node::new(0, size, Color::new(0, 0, 0, 0), std::ptr::null_mut()));

	return 1;
}

#[gmod_open]
fn open(l: LuaState) -> i32 {
	let user_data_funcs = reg![
		"Fill" => fill,
		"GetChangedData" => get_changed_data,
		"GetData" => get_data,
		"Optimize" => optimize,
		"FillSDF" => fill_sdf,
		"GetVoxels" => get_voxels,
		"Intersect" => intersect
	];

	let meta_funcs = reg![
		"__gc" => collectgarbage
	];

	// create a new metatable to attach to the userdata
	luaL_openlib(l, cstr!("MVE"), user_data_funcs.as_ptr(), 0);
	luaL_newmetatable(l, cstr!("MVE"));
	luaL_openlib(l, 0 as *const i8, meta_funcs.as_ptr(), 0);

	lua_pushstring(l, cstr!("__index"));
	lua_pushvalue(l, -3);
	lua_rawset(l, -3);

	lua_pushstring(l, cstr!("__metatable"));
	lua_pushvalue(l, -3);
	lua_rawset(l, -3);

	lua_pop(l, 2);

	// create a function that makes the userdata
	lua_register(l, cstr!("Octree"), new);
	
	printgm!(l, "Loaded MVE!");
	return 0;
}