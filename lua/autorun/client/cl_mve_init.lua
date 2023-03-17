AddCSLuaFile()

MVE.octree = MVE.octree or {}
MVE.meshes = MVE.meshes or {}
MVE.render_meshes = MVE.render_meshes or {}
local mesh_size = 32

local function vector_tostring(vec)
	return string.format("%i %i %i", vec[1], vec[2], vec[3])
end

local function string_tovector(str)
	local v = string.Split(str, " ")
	return Vector(v[1], v[2], v[3])
end

local function vector_floor(vec, cs)
	return Vector(math.floor(vec[1] * cs), math.floor(vec[2] * cs), math.floor(vec[3] * cs))
end

local min_lighting = render.GetAmbientLightColor()[1]
local function compute_lighting(normal)
	local light = normal:Dot(util.GetSunInfo().direction) * 0.3 + 0.7
	return Vector(light, light, light)
end
local function calc_lighting(p, n, r, g, b)
	local mult = compute_lighting(n) + render.ComputeDynamicLighting(p, n)	//render.ComputeLighting(p, n) * 0.5
	//mult[1] = math.max(mult[1], min_lighting)
	//mult[2] = math.max(mult[2], min_lighting)
	//mult[3] = math.max(mult[3], min_lighting)
	return math.min(r * mult[1], 255), math.min(g * mult[2], 255), math.min(b * mult[3], 255)
end

local function project_uvs(point, normal)
	local projected_point = point - normal * normal:Dot(point)
	local right = (normal[1] != 1 and normal[1] != -1) and normal:Cross(Vector(1, 0, 0)) or Vector(0, 1, 0)
	local forward = normal:Cross(right)
	local u = projected_point:Dot(right) * MVE.inv_chunk_size
	local v = projected_point:Dot(forward) * MVE.inv_chunk_size
	return u, v
end

// adds a quad using 2 triangles
local offset = Vector(16, 16, 16)
local function add_quad(p1, p2, p3, p4, normal, r, g, b)
	local r1, g1, b1 = calc_lighting(p1 + offset, normal, r, g, b)

	//local r1, g1, b1 = calc_lighting(p1, normal, r, g, b)
	mesh.Position(p1)
	mesh.Color(r1, g1, b1, 255)
	//mesh.TexCoord(0, project_uvs(p1, normal))
	mesh.AdvanceVertex()

	//local r1, g1, b1 = calc_lighting(p2, normal, r, g, b)
	mesh.Position(p2)
	mesh.Color(r1, g1, b1, 255)
	//mesh.TexCoord(0, project_uvs(p2, normal))
	mesh.AdvanceVertex()

	//local r1, g1, b1 = calc_lighting(p3, normal, r, g, b)
	mesh.Position(p3)
	mesh.Color(r1, g1, b1, 255)
	//mesh.TexCoord(0, project_uvs(p3, normal))
	mesh.AdvanceVertex()
	
	//local r1, g1, b1 = calc_lighting(p4, normal, r, g, b)
	mesh.Position(p4)
	mesh.Color(r1, g1, b1, 255)
	//mesh.TexCoord(0, project_uvs(p4, normal))
	mesh.AdvanceVertex()
end

local function get_quad_count(voxels)
	local count = 0
	for _, block in pairs(voxels) do
		local sides = block[6]
		if !sides then continue end
		if bit.band(sides, 1) == 1 then count = count + 1 end
		if bit.band(sides, 2) == 2 then count = count + 1 end
		if bit.band(sides, 4) == 4 then count = count + 1 end
		if bit.band(sides, 8) == 8 then count = count + 1 end
		if bit.band(sides, 16) == 16 then count = count + 1 end
		if bit.band(sides, 32) == 32 then count = count + 1 end
	end
	return count
end

local function build_mesh(chunk_pos)
	if MVE.meshes[chunk_pos] then 
		MVE.meshes[chunk_pos][1]:Destroy()
		MVE.meshes[chunk_pos] = nil
		MVE.render_meshes[chunk_pos] = nil
	end

	// todo: dont use table.Count
	local count = get_quad_count(MVE.octree[chunk_pos])
	if count == 0 then 
		MVE.octree[chunk_pos] = nil
		return 
	end

	MVE.meshes[chunk_pos] = {Mesh(), string_tovector(chunk_pos) * mesh_size * MVE.chunk_size}
	MVE.render_meshes[chunk_pos] = MVE.meshes[chunk_pos][1]
	local err, str
	mesh.Begin(MVE.meshes[chunk_pos][1], MATERIAL_QUADS, count)
	err, str = pcall(function()
	for _, block in pairs(MVE.octree[chunk_pos]) do
		//front
		local pos = block[1]
		local size = block[2]
		local r = block[3]
		local g = block[4]
		local b = block[5]
		local sides = block[6]
		if bit.band(sides, 1) == 1 then
			add_quad(
				pos + Vector(size, size, size), 
				pos + Vector(size, size, 0),
				pos + Vector(size, 0, 0),
				pos + Vector(size, 0, size),
				Vector(1, 0, 0),
				r, g, b
			)
		end

		//back
		if bit.band(sides, 2) == 2 then
			add_quad(
				pos + Vector(0, size, size),
				pos + Vector(0, 0, size),
				pos + Vector(0, 0, 0),
				pos + Vector(0, size, 0),
				Vector(-1, 0, 0),
				r, g, b
			)
		end

		//right
		if bit.band(sides, 4) == 4 then
			add_quad(
				pos + Vector(0, size, 0), 
				pos + Vector(size, size, 0),
				pos + Vector(size, size, size), 
				pos + Vector(0, size, size),
				Vector(0, 1, 0),
				r, g, b
			)
		end
		
		//left
		if bit.band(sides, 8) == 8 then
			add_quad(
				pos + Vector(0, 0, 0), 
				pos + Vector(0, 0, size),
				pos + Vector(size, 0, size), 
				pos + Vector(size, 0, 0),
				Vector(0, -1, 0),
				r, g, b
			)
		end

		//top
		if bit.band(sides, 16) == 16 then
			add_quad(
				pos + Vector(0, 0, size), 
				pos + Vector(0, size, size),
				pos + Vector(size, size, size), 
				pos + Vector(size, 0, size),
				Vector(0, 0, 1),
				r, g, b
			)
		end
				
		//bottom
		if bit.band(sides, 32) == 32 then
			add_quad(
				pos + Vector(0, 0, 0), 
				pos + Vector(size, 0, 0),
				pos + Vector(size, size, 0), 
				pos + Vector(0, size, 0),
				Vector(0, 0, -1),
				r, g, b
			)
		end
	end		// for loop
	end)
	mesh.End()
	if !err then print(str) end
end

local mesh_buffer = {}
net.Receive("mve_addvoxels", function(len)
	local voxel_count = 0
	local reparse_chunks = {}
	while voxel_count < len do
		local valid_voxel = net.ReadBool()
		local pos = Vector(net.ReadInt(10), net.ReadInt(10), net.ReadInt(10)) * MVE.chunk_size
		local str_pos = vector_tostring(pos)
		local chunk_pos = vector_tostring(vector_floor(pos, MVE.inv_chunk_size / mesh_size))
		voxel_count = voxel_count + 31
		MVE.octree[chunk_pos] = MVE.octree[chunk_pos] or {}

		if valid_voxel then
			local r, g, b, a = net.ReadUInt(8), net.ReadUInt(8), net.ReadUInt(8)
			local sides = net.ReadUInt(6)
			local size = MVE.chunk_size * bit.lshift(1, net.ReadUInt(4))
			voxel_count = voxel_count + 34

			MVE.octree[chunk_pos][str_pos] = {pos, size, r, g, b, sides}	// add voxel to table
		elseif MVE.octree[chunk_pos][str_pos] then // remove voxel from table
			table.Empty(MVE.octree[chunk_pos][str_pos])
			MVE.octree[chunk_pos][str_pos] = nil
		end

		mesh_buffer[chunk_pos] = true
	end

	//for chunk_pos, _ in pairs(reparse_chunks) do
	//	table.insert(mesh_buffer, chunk_pos)
	//	//build_mesh(chunk_pos)
	//end
end)

hook.Add("Think", "mve_meshbuffer", function()
	for i = 1, 10 do
		local chunk_pos = next(mesh_buffer)
		if !chunk_pos then 
			return 
		end
		
		mesh_buffer[chunk_pos] = nil

		if MVE.octree[chunk_pos] then
			build_mesh(chunk_pos)
		end
	end
end)

local mat = CreateMaterial("mve_voxels16", "UnlitGeneric", {
	["$vertexcolor"] = 1,
	//["$basetexture"] = "phoenix_storms/stripes",
	//["$vertexalpha"] = 1,
	//["$alphatest"] = 1,
	//["$translucent"] = 1,
	["$model"] = 1,
})

local size = mesh_size * MVE.chunk_size
local box_points = {
	Vector(0, 0, 0),
	Vector(0, 0, size),
	Vector(0, size, 0),
	Vector(0, size, size),
	Vector(size, 0, 0),
	Vector(size, 0, size),
	Vector(size, size, 0),
	Vector(size, size, size),
}

local math_abs = math.abs
local function box_plane_intersect(box_pos, plane_pos, plane_normal) 
	for i = 1, 8 do
		local point_pos = box_points[i] + box_pos
		local point_dist = (point_pos - plane_pos):Dot(plane_normal)
		if point_dist > 0 then	//and point_dist < 15000
			return true
		end

		// too far to be considered
		if point_dist < -size * 2 then
			return false
		end
	end
	return false
end

local last_key
hook.Add("PostDrawOpaqueRenderables", "mve_voxels", function(_, sky, sky2)
	if sky2 then return end
	render.SetMaterial(mat)
	
	for _, mesh in pairs(MVE.render_meshes) do
		mesh:Draw()
	end
	
	local eyepos = LocalPlayer():EyePos()
	local forward = LocalPlayer():EyeAngles():Forward()

	for i = 1, 128 do
		local mesh
		last_key, mesh = next(MVE.meshes, last_key)
		if !last_key then break end

		if box_plane_intersect(mesh[2], eyepos, forward) then
			MVE.render_meshes[last_key] = MVE.render_meshes[last_key] or mesh[1]
		else
			MVE.render_meshes[last_key] = nil
		end
	end
end)

//hook.Remove("PostDrawTranslucentRenderables", "mve_voxels")
