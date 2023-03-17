use rglua::prelude::*;
use glam::*;
use crate::col::*;

include!("node_helpers.rs");

const MIN_NODE_SIZE : u8 = 0;

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
	pos : u8,
	pub size : u8,
	color : Color,
	parent : *mut Node,
	changed : u8,// 000 = no change, 001 = color changed / new node, 010 = sides changed, 100 = octree just merged
	sides : u8,
	pub children : Option<[Box::<Node>; 8]>,
}

unsafe impl Send for Node {}

impl Node {
	pub fn new(pos : u8, size : u8, color : Color, parent : *mut Node) -> Node {
		return Node {
			pos,
			size,
			color,
			parent,
			changed : 0b001,
			sides : 0b00111111,
			children : None,
		};
	}

	pub fn split(&mut self) {
		// dont split an already split node
		if self.children.is_some() {
			return;
		}

		let split_size_u8 = self.size - 1;
		let self_pointer = self as *mut Node;
		
		// BIRTH THE CHILDREN
		self.children = Some([
			Box::new(Node::new(0, split_size_u8, self.color, self_pointer)),
			Box::new(Node::new(1, split_size_u8, self.color, self_pointer)),
			Box::new(Node::new(2, split_size_u8, self.color, self_pointer)),
			Box::new(Node::new(3, split_size_u8, self.color, self_pointer)),
			Box::new(Node::new(4, split_size_u8, self.color, self_pointer)),
			Box::new(Node::new(5, split_size_u8, self.color, self_pointer)),
			Box::new(Node::new(6, split_size_u8, self.color, self_pointer)),
			Box::new(Node::new(7, split_size_u8, self.color, self_pointer)),
		]);
	}

	// lazy overload functions
	pub fn get_offset(&self, offset : IVec3) -> IVec3 {
		return get_offset(self.pos, self.size, offset);
	}

	fn get_anti_offset(&self, offset : IVec3) -> IVec3 {
		return get_anti_offset(self.pos, self.size, offset);
	}

	// returns true if changed, false if not
	pub fn fill(&mut self, pos : IVec3, target_data : Color, offset : IVec3) -> u8 {
		// dont bother updating nodes if its the same color
		if self.children.is_none() && self.color == target_data {
			return 0b000;
		}

		// if the position is outside the node, return
		let world_size = (1 << self.size) as i32;
		if pos.x < offset.x || pos.x >= offset.x + world_size || 
		   pos.y < offset.y || pos.y >= offset.y + world_size || 
		   pos.z < offset.z || pos.z >= offset.z + world_size {
			return 0b000;
		}

		// 0 is the smallest size
		if self.size == MIN_NODE_SIZE {
			self.color = target_data;
			self.changed |= 0b001;
			return 0b001;
		}

		self.split();

		//if self.children.is_some() {
			for child in self.children.as_mut().unwrap() {
				self.changed |= child.fill(pos, target_data, child.get_offset(offset));
			}
		//}

		return self.changed;
	}

	pub fn fill_sdf(&mut self, offset : IVec3) -> u8 {
		let size = (1 << self.size) as f32 / 2.0;
		let world_offset = vec3(offset.x as f32 + size, offset.y as f32 + size, offset.z as f32 + size);
		let distance = generate_sdf(world_offset);

		let area = size * SQRT3;
		if -distance > area && self.size < 6 {
			//if self.color == target_data {
			//	return 0b000;
			//}

			if distance > -2.0 {
				self.color = Color::new(0, rand::random::<u8>() / 10 + 230, 0, 255);
			}
			else if distance > -100.0 {
				self.color = Color::new(rand::random::<u8>() / 10 + 100, 50, 0, 255);
			}
			else {
				self.color = Color::new(30, 30, 30, 255);
			}

			self.children = None;
			self.changed = 0b001;
			return 0b001;
		}
		
		if area > distance && self.size > MIN_NODE_SIZE {
			self.split();

			for child in self.children.as_mut().unwrap() {
				self.changed |= child.fill_sdf(child.get_offset(offset));
			}
		}

		return self.changed;
	}

	// combine octrees with the same color
	pub fn optimize_changed(&mut self, l : LuaState, offset : IVec3) {
		if self.changed & 0b001 == 0 {
			return;
		}

		if self.children.is_some() {
			let mut_childs = self.children.as_mut().unwrap();
			let mut last_color = mut_childs[0].color;

			let mut same = true;
			for i in 0 .. 8 {
				let child = &mut mut_childs[i];
				child.optimize_changed(l, child.get_offset(offset));

				// child is different, bail
				if child.children.is_some() || (i != 0 && child.color != last_color) {
					same = false;
				}
				
				last_color = child.color;
			}

			// if we get here, all children are the same color and we can merge
			if same {
				let ref_childs = self.children.as_ref().unwrap();
				for i in 1 .. 8 {	// dont include first child, because it is already included in the parent
					if ref_childs[i].sides == 0 {continue}	// already invalid
					push_invalid_pos_data(l, ref_childs[i].get_offset(offset));
				}

				// merge sides from children to main node
				self.sides &= 0b00111110 | (ref_childs[1].sides | ref_childs[3].sides | ref_childs[5].sides | ref_childs[7].sides);
				self.sides &= 0b00111101 | (ref_childs[0].sides | ref_childs[2].sides | ref_childs[4].sides | ref_childs[6].sides);
				self.sides &= 0b00111011 | (ref_childs[2].sides | ref_childs[3].sides | ref_childs[6].sides | ref_childs[7].sides);
				self.sides &= 0b00110111 | (ref_childs[0].sides | ref_childs[1].sides | ref_childs[4].sides | ref_childs[5].sides);
				self.sides &= 0b00101111 | (ref_childs[4].sides | ref_childs[5].sides | ref_childs[6].sides | ref_childs[7].sides);
				self.sides &= 0b00011111 | (ref_childs[0].sides | ref_childs[1].sides | ref_childs[2].sides | ref_childs[3].sides);

				self.color = last_color;
				self.children = None;
				self.changed |= 0b100;
			}
		}
	}

	pub fn node_at_pos_down(&self, pos : IVec3, offset : IVec3) -> *mut Node {
		// position isnt inside this node, bail
		let world_size = (1 << self.size) as i32;
		if pos.x < offset.x || pos.x >= offset.x + world_size || 
		   pos.y < offset.y || pos.y >= offset.y + world_size || 
		   pos.z < offset.z || pos.z >= offset.z + world_size {
			return std::ptr::null_mut();
		}

		// travel down
		if self.children.is_some() {
			for child in self.children.as_ref().unwrap() {
				let result = child.node_at_pos_down(pos, child.get_offset(offset));
				if !result.is_null() {
					return result;
				}
			}

			// should never be run
			return std::ptr::null_mut();
		}
		else {
			return self as *const Node as *mut Node;
		}
	}

	// goes up the stack and back down to find node
	pub fn node_at_pos_up(&self, pos : IVec3, offset : IVec3) -> *mut Node {
		// if the position isnt inside this node, go up tree
		let world_size = (1 << self.size) as i32;
		if pos.x < offset.x || pos.x >= offset.x + world_size || 
		   pos.y < offset.y || pos.y >= offset.y + world_size || 
		   pos.z < offset.z || pos.z >= offset.z + world_size {
			if !self.parent.is_null() {
				return unsafe {(*self.parent).node_at_pos_up(pos, self.get_anti_offset(offset))};
			}

			return std::ptr::null_mut();
		}

		// position is inside this node, go down tree
		return self.node_at_pos_down(pos, offset);
	}

	pub fn intersect(&self, ray_pos : Vec3, ray_dir : Vec3, offset : IVec3) -> f32 {
		// initial check
		let world_size = (1 << self.size) as f32;
		let offset2 = offset.as_vec3();
		let intersect = intersect_box(ray_pos, ray_dir, offset2, offset2 + vec3(world_size, world_size, world_size));
		if intersect == f32::INFINITY {
			return intersect;
		}

		// we know the ray will intersect, now do recursive checks
		if self.children.is_some() {
			let mut min_dist = f32::INFINITY;
			for child in self.children.as_ref().unwrap() {
				min_dist = child.intersect(ray_pos, ray_dir, child.get_offset(offset)).min(min_dist);
			}
			return min_dist;
		}
		else if self.color.a > 0 {
			return intersect;
		}
		else {
			return f32::INFINITY;
		}
	}

	// goes up the node tree and changes side value
	pub fn change_sides_recursive(&mut self, bit : u8) {
		if self.changed > 0 {
			return;
		}

		self.changed = bit;
		if !self.parent.is_null() {
			unsafe {(*self.parent).change_sides_recursive(bit)};
		}
	}

	pub fn cull_changed(&mut self, offset : IVec3) {
		if self.changed & 0b001 == 0 {
			return;
		}

		if self.children.is_some() {
			for child in self.children.as_mut().unwrap() {
				child.cull_changed(child.get_offset(offset));
			}
		} else {
			let size = 1 << self.size;
			cull_nodes(self, offset,  ivec3(size, 0, 0), 0b00000001, 0b00000010);
			cull_nodes(self, offset, -ivec3(1,    0, 0), 0b00000010, 0b00000001);
			cull_nodes(self, offset,  ivec3(0, size, 0), 0b00000100, 0b00001000);
			cull_nodes(self, offset, -ivec3(0, 1,    0), 0b00001000, 0b00000100);
			cull_nodes(self, offset,  ivec3(0, 0, size), 0b00010000, 0b00100000);
			cull_nodes(self, offset, -ivec3(0, 0,    1), 0b00100000, 0b00010000);
		}
	}

	pub fn cull_merged(&mut self, offset : IVec3) {
		if self.changed & 0b001 == 0 {
			return;
		}

		if self.children.is_some() {
			for child in self.children.as_mut().unwrap() {
				child.cull_merged(child.get_offset(offset));
			}
		} else if self.changed & 0b100 != 0 {
			let size = 1 << self.size;
			cull_side_nodes(self, offset,  ivec3(size, 0, 0), 0b00000010);
			cull_side_nodes(self, offset, -ivec3(1,    0, 0), 0b00000001);
			cull_side_nodes(self, offset,  ivec3(0, size, 0), 0b00001000);
			cull_side_nodes(self, offset, -ivec3(0, 1,    0), 0b00000100);
			cull_side_nodes(self, offset,  ivec3(0, 0, size), 0b00100000);
			cull_side_nodes(self, offset, -ivec3(0, 0,    1), 0b00010000);
		}
	}

	pub fn get_changed_data(&mut self, l : LuaState, offset : IVec3, sort : bool) {
		if self.changed == 0 {
			return;
		}

		self.changed = 0b000;

		if self.children.is_some() {
			for child in self.children.as_mut().unwrap() {
				child.get_changed_data(l, child.get_offset(offset), sort);
			}
		} else if !sort || (self.color.a > 0 && self.sides != 0) {
			self.push_lua_data(l, offset);
			lua_rawseti(l, -2, lua_objlen(l, -2) as i32 + 1);
		}
	}

	pub fn get_data(&self, l : LuaState, offset : IVec3) {
		if self.children.is_some() {
			for child in self.children.as_ref().unwrap() {
				child.get_data(l, child.get_offset(offset));
			}
		} else if self.color.a > 0 && self.sides != 0 {
			self.push_lua_data(l, offset);
			lua_rawseti(l, -2, lua_objlen(l, -2) as i32 + 1);
		}
	}

	pub fn push_lua_data(&self, l : LuaState, offset : IVec3) {
		// table to put data inside
		lua_newtable(l);	// {}	

		// push position to newly created table
		lua_pushvector(l, Vector::new(offset.x as f32, offset.y as f32, offset.z as f32));
		lua_rawseti(l, -2, 1);

		// push color
		lua_pushnumber(l, self.color.r as LuaNumber);
		lua_rawseti(l, -2, 2);	

		lua_pushnumber(l, self.color.g as LuaNumber);
		lua_rawseti(l, -2, 3);	

		lua_pushnumber(l, self.color.b as LuaNumber);
		lua_rawseti(l, -2, 4);

		lua_pushnumber(l, self.color.a as LuaNumber);
		lua_rawseti(l, -2, 5);

		// push sides
		lua_pushnumber(l, self.sides as LuaNumber);
		lua_rawseti(l, -2, 6);
		
		// push size
		lua_pushnumber(l, self.size as LuaNumber);
		lua_rawseti(l, -2, 7);	// {pos, size}
	}
}
