// creates a new octree and puts it on the stack
fn push_octree(l: LuaState, octree: &Node) {
	let octree_userdata = lua_newuserdata(l, std::mem::size_of::<Node>()) as *mut Node;
	unsafe {*octree_userdata = octree.clone()}	// apply octree data to new empty userdata
	luaL_getmetatable(l, cstr!("MVE"));
	lua_setmetatable(l, -2);
}

fn check_octree(l: LuaState, index: i32) -> *mut Node {
	let octree_userdata = lua_touserdata(l, index) as *mut Node;
	if octree_userdata.is_null() {
		luaL_typerror(l, index, cstr!("MVE"));
	}
	return octree_userdata;
}

fn lua_checkoctree<'a>(l: LuaState, index: i32) -> &'a mut Node {
	let octree_userdata = check_octree(l, index);
	unsafe {
		return &mut *octree_userdata;
	}
}

fn vector_2_ivec3(vec: Vector) -> IVec3 {
	return IVec3::new(vec.x.floor() as i32, vec.y.floor() as i32, vec.z.floor() as i32);
}

fn vector_2_vec3(vec: Vector) -> Vec3 {
	return Vec3::new(vec.x, vec.y, vec.z);
}