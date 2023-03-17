require("voxels")
util.AddNetworkString("mve_addvoxels")

MVE.octree = MVE.octree or Octree(10)
MVE.buffer = MVE.buffer or {}

local net_size = 800
local table_pos = 1
local function buffer_voxels()
	if !MVE.buffer[1] then return end
	// add voxel
	net.Start("mve_addvoxels")
	for i = 1, net_size do
		local data = MVE.buffer[table_pos]	// 1=pos,2=r,3=g,4=b,5=a,6=sides,7=size
		table_pos = table_pos + 1
		if !data then 
			table_pos = 1
			table.Empty(MVE.buffer)
			break 
		end
		local valid = data[5] and data[5] > 0 and data[6] != 0
		net.WriteBool(valid)
		net.WriteInt(data[1][1], 10)	// pos x
		net.WriteInt(data[1][2], 10)	// pos y
		net.WriteInt(data[1][3], 10)	// pos z
		if valid then
			net.WriteUInt(data[2], 8)	// color r
			net.WriteUInt(data[3], 8)	// color g
			net.WriteUInt(data[4], 8)	// color b
			//net.WriteUInt(data[5], 8)	// color a
			net.WriteUInt(data[6], 6)	// sides
			net.WriteUInt(data[7], 4)	// size
		end
	end
	net.Broadcast()
end	
hook.Add("Think", "mve_buffer", buffer_voxels)