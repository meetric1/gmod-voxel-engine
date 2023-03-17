
local function floor_vector(vec)
	return Vector(math.floor(vec[1]), math.floor(vec[2]), math.floor(vec[3]))
end

local function create_collider(pos)
	local ent = ents.Create("prop_physics")
	ent:SetModel("models/hunter/blocks/cube05x05x05.mdl")
	ent:SetPos(pos)
	ent:SetAngles(Angle())
	ent:Spawn()
	//ent:SetSolid(SOLID_VPHYSICS)
	//ent:SetMoveType(MOVETYPE_NONE)
	//ent:Activate()
	ent:GetPhysicsObject():EnableMotion(false)
	ent:SetNoDraw(true)
	return ent
end

local offset = Vector(0.5, 0.5, 0.5) * MVE.chunk_size
function MVE.update_collision(ent, pos) 
	pos = pos or floor_vector(ent:GetPos() * MVE.inv_chunk_size)

	ent.COLLIDERS = ent.COLLIDERS or {}
	local i = 0
	for x = -1, 1 do
		for y = -1, 1 do
			for z = -1, 3 do
				i = i + 1
				local floored_pos = pos + Vector(x, y, z)
				local data = MVE.octree:GetVoxels(floored_pos)
				local valid = data and data[5] != 0 and data[6] != 0
				
				if IsValid(ent.COLLIDERS[i]) then
					if valid then
						ent.COLLIDERS[i]:SetPos(floored_pos * MVE.chunk_size + offset)
						//ent.COLLIDERS[i]:Remove()
						//ent.COLLIDERS[i] = create_collider(floored_pos * MVE.chunk_size + offset)
					else
						ent.COLLIDERS[i]:Remove()
						ent.COLLIDERS[i] = nil
					end
				else
					if valid then
						ent.COLLIDERS[i] = create_collider(floored_pos * MVE.chunk_size + offset)
					end
				end
				
				//debugoverlay.Box(floored_pos * MVE.chunk_size, Vector(0, 0, 0), Vector(MVE.chunk_size, MVE.chunk_size, MVE.chunk_size), 1, Color(255, 0, 0, 0))
			end
		end
	end
end

hook.Add("PlayerTick", "voxel_collision", function()
	if !MVE.octree then return end
	for k, ent in ipairs(player.GetAll()) do
		local new_pos = floor_vector(ent:GetPos() * MVE.inv_chunk_size)
		if ent.COLLIDER_POS == new_pos then return end
		ent.COLLIDER_POS = new_pos
		MVE.update_collision(ent, new_pos)
	end
end)