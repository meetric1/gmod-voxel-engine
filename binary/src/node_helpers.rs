// invalid positions to be destroyed on client
fn push_invalid_pos_data(l : LuaState, offset : IVec3) {
	// table to put data inside
	lua_newtable(l);	// {}	

	// push position to newly created table
	lua_pushvector(l, Vector::new(offset.x as f32, offset.y as f32, offset.z as f32));
	lua_rawseti(l, -2, 1);
	lua_rawseti(l, -2, lua_objlen(l, -2) as i32 + 1);
}

fn get_offset(pos : u8, size : u8, offset : IVec3) -> IVec3 {
	let x = ( pos       & 1) as i32;
	let y = ((pos >> 1) & 1) as i32;
	let z = ((pos >> 2) & 1) as i32;
	return offset + ivec3(x, y, z) * (1 << size);
}

fn get_anti_offset(pos : u8, size : u8, offset : IVec3) -> IVec3 {
	let x = ( pos       & 1) as i32;
	let y = ((pos >> 1) & 1) as i32;
	let z = ((pos >> 2) & 1) as i32;
	return offset - ivec3(x, y, z) * (1 << size);
}

fn cull_nodes(main_node : &mut Node, offset : IVec3, pos : IVec3, bit_a : u8, bit_b : u8) {
	let check = main_node.node_at_pos_up(offset + pos, offset);
	if check.is_null() {
		main_node.sides &= !bit_a; // side doesnt exist, just remove
		return;
	}

	// shouldn't crash since we check for null, just hope we dont have a solar flare
	let secondary_node = unsafe { &mut *check };

	if main_node.color.a > 0 {
		if secondary_node.color.a > 0 {
			if main_node.size <= secondary_node.size {
				main_node.sides &= !bit_a;	// remove side

				// if secondary node is changed its gonna be iterated over anyways so dont bother
				// also check if secondary node already has this side removed
				if secondary_node.changed & 0b001 == 0 && secondary_node.sides & bit_b == bit_b {
					if secondary_node.size <= main_node.size {
						secondary_node.sides &= !bit_b; 
						secondary_node.change_sides_recursive(0b010);
					}
				}
			}
		}
		else {
			main_node.sides |= bit_a; // add side
		}
	}
	else {
		// add side for secondary node
		if secondary_node.color.a > 0 {
			if secondary_node.changed & 0b001 == 0 && secondary_node.sides & bit_b == 0 {
				secondary_node.sides |= bit_b; 
				secondary_node.change_sides_recursive(0b010);
			}
		}
	}
}

fn cull_side_nodes(main_node : &mut Node, offset : IVec3, pos : IVec3, bit_b : u8) {
	let check = main_node.node_at_pos_up(offset + pos, offset);
	if check.is_null() {
		return;
	}

	// shouldn't crash since we check for null, just hope we dont have a solar flare
	let secondary_node = unsafe { &mut *check };

	if main_node.color.a > 0 && secondary_node.color.a > 0 {
		if main_node.size <= secondary_node.size {
			// if secondary node is changed its gonna be iterated over anyways so dont bother
			// also check if secondary node already has this side removed
			if secondary_node.sides & bit_b == bit_b {
				if secondary_node.size <= main_node.size {
					secondary_node.sides &= !bit_b; 
					secondary_node.change_sides_recursive(0b010);
				}
			}
		}
	}
}

fn generate_sdf(offset : Vec3) -> f32 {
	//return offset.z + ((offset.x / 100.0).sin() * 100.0 + (offset.y / 100.0).cos() * 100.0).min(0.0);//offset.length() - 100.0;
	return if offset.z > 0.0 {vec3(offset.x, offset.y, 100.0).length() - 100.0} else {vec3(offset.x, offset.y, 0.0).length() - 100.0};
}

fn intersect_box(ray_pos : Vec3, ray_dir : Vec3, min : Vec3, max : Vec3) -> f32 {
    let tmin = (min - ray_pos) * ray_dir;
    let tmax = (max - ray_pos) * ray_dir;
    let t1 = tmin.min(tmax);
    let t2 = tmin.max(tmax);
    let tnear = t1.x.max(t1.y).max(t1.z);
    let tfar = t2.x.min(t2.y).min(t2.z);
    return if tnear < tfar && tfar > 0.0 {tnear} else {f32::INFINITY};
}

const SQRT3 : f32 = 1.7320508075688772935274463415059;