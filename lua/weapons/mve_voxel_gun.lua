SWEP.Base = "weapon_base"
SWEP.PrintName = "Voxel Gun"

SWEP.ViewModel = "models/weapons/c_pistol.mdl"
SWEP.ViewModelFlip = false
SWEP.UseHands = true

SWEP.WorldModel = "models/weapons/w_pistol.mdl"
SWEP.SetHoldType = "pistol"

SWEP.Weight = 5
SWEP.AutoSwichTo = true
SWEP.AutoSwichFrom = false

SWEP.Category = "MVE"
SWEP.Slot = 0
SWEP.SlotPos = 1

SWEP.DrawAmmo = true
SWEP.DrawChrosshair = true

SWEP.Spawnable = true
SWEP.AdminSpawnable = false

SWEP.Primary.ClipSize = -1
SWEP.Primary.DefaultClip = -1
SWEP.Primary.Ammo = "none"
SWEP.Primary.Automatic = true

SWEP.Secondary.ClipSize = -1
SWEP.Secondary.DefaultClip = -1
SWEP.Secondary.Ammo = "none"
SWEP.Secondary.Automatic = false

if CLIENT then return end

local function add_voxels(octree)
	octree:Optimize(MVE.buffer)
	octree:GetChangedData(MVE.buffer, 0)
end

local function vector_floor(vec, cs)
	return Vector(math.floor(vec[1] * cs), math.floor(vec[2] * cs), math.floor(vec[3] * cs))
end

// remove blocks
function SWEP:PrimaryAttack()
	if CLIENT then return end
	local owner = self.Owner
	local pos = owner:EyePos()
	local dir = owner:EyeAngles():Forward()
	local hit_pos = pos + dir * MVE.octree:Intersect(pos * MVE.inv_chunk_size, dir) * MVE.chunk_size + dir * 0.1
	MVE.octree:Fill(hit_pos * MVE.inv_chunk_size, 255, 255, 255, 0)

	add_voxels(MVE.octree)
	MVE.update_collision(self.Owner)

	self:SetNextPrimaryFire(CurTime() + 0.25)
end

// place blocks
function SWEP:SecondaryAttack()
	local owner = self.Owner
	local pos = owner:EyePos()
	local dir = owner:EyeAngles():Forward()
	local hit_pos = pos + dir * MVE.octree:Intersect(pos * MVE.inv_chunk_size, dir) * MVE.chunk_size - dir * 0.1

	local floored_pos = vector_floor(hit_pos, MVE.inv_chunk_size) * MVE.chunk_size
	local intersecting = util.TraceHull({
		start = floored_pos,
		endpos = floored_pos,
		mins = Vector(MVE.chunk_size, MVE.chunk_size, MVE.chunk_size * 0.9) * 0.1,
		maxs = Vector(MVE.chunk_size, MVE.chunk_size, MVE.chunk_size * 0.9) * 0.9,
	}).Hit

	if !intersecting then
		MVE.octree:Fill(hit_pos * MVE.inv_chunk_size, 255, 255, 255, 255)
	end

	add_voxels(MVE.octree)
	MVE.update_collision(self.Owner)

	self.SECONDARY_INIT = CurTime() + 0.25
	//self:SetNextSecondaryFire(CurTime() + 0.25)
end

function SWEP:Think() 
	if !self.SECONDARY_INIT then return end
	if self.Owner:KeyDown(IN_ATTACK2) then
		if CurTime() > self.SECONDARY_INIT then
			self:SecondaryAttack()
			self.SECONDARY_INIT = CurTime() + 0.25
		end
	else
		self.SECONDARY_INIT = nil
	end
end

local can = true
function SWEP:Reload()
	if CLIENT then return end
	if !can then return end
	MVE.octree:FillSDF()
	MVE.octree:GetChangedData(MVE.buffer, 1)
	can = false
end