
AddCSLuaFile()

ENT.Type = "anim"
ENT.Base = "base_gmodentity"

ENT.Category     = "MVE"
ENT.PrintName    = "chunk"
ENT.Author       = "Mee"
ENT.Purpose      = ""
ENT.Instructions = ""
ENT.Spawnable    = true

local function generate_verts(pos)
	local scale = MVE.chunk_size
	local pos = pos * scale
	return {
		Vector(0, 0, 0) * scale + pos,
		Vector(0, 0, 1) * scale + pos,
		Vector(0, 1, 0) * scale + pos,
		Vector(0, 1, 1) * scale + pos,
		Vector(1, 0, 0) * scale + pos,
		Vector(1, 0, 1) * scale + pos,
		Vector(1, 1, 0) * scale + pos,
		Vector(1, 1, 1) * scale + pos
	}
end

function ENT:InitCollision(pos)
	local verts = {}
	for _, block in ipairs(MVE.octree:GetVoxels(pos, 2)) do
		if block[5] == 0 then continue end	// is air
		local vert_pos = block[1]
		table.insert(verts, generate_verts(vert_pos - pos))
		//debugoverlay.Box(vert_pos * MVE.chunk_size, Vector(), Vector(1, 1, 1) * MVE.chunk_size, 1, Color(255, 0, 0, 0))
	end
	self:SetPos(pos * MVE.chunk_size)
	self:PhysicsDestroy()
	//self:PhysicsInitMultiConvex(verts)
	local phys = self:GetPhysicsObject()
	if phys:IsValid() then
		phys:EnableMotion(false)
		phys:SetPos(self:GetPos())
		phys:SetAngles(self:GetAngles())
	end
end

function ENT:Initialize()
	if CLIENT then return end
	self:SetModel("models/props_junk/garbage_milkcarton002a.mdl")
	self:SetMoveType(MOVETYPE_NONE)
	self:SetSolid(SOLID_VPHYSICS)
	self:EnableCustomCollisions(true)
	//self:GetPhysicsObject():EnableMotion(false)
	self:SetPos(Vector())
	self:SetAngles(Angle())
end

function ENT:Think() 
	if CLIENT then return end
	debugoverlay.Box(self:GetPos(), Vector(), Vector(1, 1, 1) * MVE.chunk_size, 0.2, Color(0, 0, 255, 0))

	local pos = Entity(1):GetPos() * MVE.inv_chunk_size
	local floored_pos = Vector(math.floor(pos[1]), math.floor(pos[2]), math.floor(pos[3]))
	if self:GetPos() * MVE.inv_chunk_size != floored_pos then
		self:InitCollision(floored_pos)
		print("rebuilding collision")
	end
	
	self:NextThink(CurTime() + 0.1)
	return true
end