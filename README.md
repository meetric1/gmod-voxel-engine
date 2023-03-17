# gmod-voxel-engine

### An unfinished voxel engine for gmod written in primarily rust
This project has been abandoned because I am unsure how to handle collision for entities larger than a player, I then wanted to make a server like 2b2t,
but alas I could not find a good way to host it. Do what you want with this source code.

### How to use:
1. goto releases tab & install v1.0
2. launch 64bit branch of gmod
3. join a world (gm_mvetest preferably)
4. get out the voxel gun and press "R" (reload) this spawns the world in (I think its a cylinder at the moment)
5. if all works correctly you should be able to place and mine blocks

### Problems you will encounter
1. It does work in multiplayer but the world isnt networked on join
2. The voxel gun cannot place on anything other than voxels
3. The voxel gun can only place white voxels cus I never added a selection editor
4. This is extremely unfinished and likely will be buggy / unoptimized (I think it has a memory leak?) 
