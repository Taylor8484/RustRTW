Function LoadLevel(FileName$)

	For i=0 To 3
		ElectroExists(i)=False
	Next
	
	MapLevel$=""

	File=MyReadFile("levels\"+FileName$,False)
	MapName$=ReadString$(File) 	; actually jsut the "level file" message
	MapName$=ReadString$(File)	; the file name (not used)
	MapName$=ReadString$(File)  ; the actual description
	MapName$=MapLevel$+MapName$
	MapTime=ReadInt(File)
	If MenuStyle=-1
		MapStyle=ReadInt(File)
	Else 
		MapStyle=ReadInt(File)
		MapStyle=MenuStyle
	EndIf
	If MenuBkg=-1
		MapBackGround=ReadInt(file)
	Else
		MapBackGround=ReadInt(file)
		MapBackGround=MenuBkg
	EndIf
	
	
	
	MapWidth=ReadInt(File)
	MapHeight=ReadInt(File)
	
	
	For i=0 To 4
		SignX(i)=-1
		SignY(i)=-1
	Next	

	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			MapTile(i,j)=ReadInt(File)
			
			If MapTile(i,j)/100=8
				;cannons - created from tiles
				If (MapTile(i,j)-800)/4 = 0 
					CreateCannon(i,j,(MapTile(i,j) Mod 4)+1,0,-1)
				Else If (MapTile(i,j)-800)/4 = 1
					CreateCannon(i,j,(MapTile(i,j) Mod 4)+1,1,0)
				Else If (MapTile(i,j)-800)/4 = 2
					CreateCannon(i,j,(MapTile(i,j) Mod 4)+1,0,1)
				Else If (MapTile(i,j)-800)/4 = 3
					CreateCannon(i,j,(MapTile(i,j) Mod 4)+1,-1,0)
				EndIf
			EndIf
			If MapTile(i,j)/100=10
				;gates
				;If MapTile(i,j) Mod 100 <2
					; laser gate / iron gate
					CreateLaserGate(i,j,(MapTile(i,j) Mod 100)/2,MapTile(i,j) Mod 2)
				;EndIf
			EndIf
			If MapTile(i,j)/100=11
				;buttons -created from tiles
				CreateButton(i,j,MapTile(i,j)-1100)
			EndIf
			If MapTile(i,j)/100=13
				;Signs
				SignX(MapTile(i,j)-1300)=i
				SignY(MapTile(i,j)-1300)=j
				MapTile(i,j)=1300
			EndIf
		Next
	Next
	
	
	; Create Electros
	;electro tiles - created here and changed into floor later
	CreateElectro()		
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)/100=7 ; electro
				MapTile(i,j)=100
			EndIf
		Next
	Next

	Snow=False
	; Create Objects
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			Obj=ReadInt(File)
			If Obj=1
				CreateStinky(i,j)
			Else If Obj=2
				CreateLoof(i,j)
			Else If Obj=3
				CreateBox(i,j,0)	;wood
			Else If Obj=4
				CreateBox(i,j,1)	;metal
			Else If Obj=5
				CreateItem(i,j,1)	;gate
			Else If Obj=6
				CreateItem(i,j,0)	;crystal
			Else If Obj=7
				CreateMonster(i,j,0,0);coily
			Else If obj>=8 And obj<16
				CreateMonster(i,j,obj Mod 2+1,(obj-8)/2);robot 
			Else If obj>=16 And obj<24
				CreateMonster(i,j,((obj+1) Mod 2)+3,(obj-16)/2);bomber
			Else If Obj=24
				CreateItem(i,j,2)	;time
			Else If Obj=25
				CreateItem(i,j,3)	;money
			Else If Obj=26
				CreateFish(i,j)		;fish
				
			Else If Obj=27
									;snow
				Snow=True
				
			Else If Obj=28 Or Obj=29 Or Obj=30
				CreatePillar(i,j,Obj-28)	;pillar
							
			EndIf
			
		Next
	Next
	
	
	
	
	For i=0 To 4
		SignText$(i)=ReadString$(File)
	Next
	
	CloseFile(File)
	
	
	CurrentLevel2=CurrentLevel
	If Version<10 And CurrentLevel2>=10
		CurrentLevel2=CurrentLevel2-5
	EndIf
	; Different Language Signs
	If Language=1
		Restore GermanText		
		
		For j=0 To CurrentLevel2
			If j<100 Or j>399
				
				Read MapName$
				For i=0 To 4
					Read SignText$(i)
				Next
			EndIf
		Next
		
		
	EndIf
	If Language=2
		Restore FrenchText		
		
		For j=0 To CurrentLevel2
			If j<100 Or j>399
				
				Read MapName$
				For i=0 To 4
					Read SignText$(i)
				Next
			EndIf
		Next
		
		
	EndIf

	
	
		
	; do a backup of the maptiles
	For i=0 To MapWidth-1
		For j=0 To MapHeight-1
			MapTile2(i,j)=MapTile(i,j)
		Next 
	Next
		
	While MouseDown(1)=True
	Wend
	
	; and initialize variables
	ConveyorActive=True
	NofTotalCrystals=NofCrystals
	
	;check if any buttons are currently depressed
	CheckButtons()
	
	; Signs
	SignActive=0
	For i=0 To 4
		If StinkyX=SignX(i) And StinkyY=SignY(i)
			StartSign(SignText$(i),8,11+i,1)
		EndIf
	Next 
	

	
	LoadStyleTextures()


End Function

Function ConstructLevel()

	; Setup the Basics
	LevelMesh=CreateMesh()
	
	
	NofLevelSurfaces=0
	
	; Floors
	; For Floors, also add all cannons/gates/buttons since they have floor underneath
	For i=0 To MapWidth-1
		For j=0 To MapHeight-1
			If MapTile(i,j)/100=8 Or MapTile(i,j)/100=10 Or MapTile(i,j)/100=11 Then MapTile(i,j)=100
		Next 
	Next
	LevelAddFloor(100,FloorTexture)
	LevelAddFloor(101,FloorTexture1)
	LevelAddFloor(102,FloorTexture2)
	LevelAddFloor(103,FloorTexture3)
	LevelAddFloor(104,FloorTexture4)
	
	
	
	
	
	LevelAddFloor(1300,FloorTextureSign)

	; and restore level data
	For i=0 To MapWidth-1
		For j=0 To MapHeight-1
			MapTile(i,j)=MapTile2(i,j)
		Next 
	Next

	
		
	; Walls
	LevelAddWall(200,WallTextureTop1,WallTextureSide1a,WallTextureSide1b)
	LevelAddDeepHole(201,WallTextureSide2b,WallTextureSide2a)
	
	; Water (only one sub-type)
	; For water, also add water breakaway tiles since they have water underneath
	For i=0 To MapWidth-1
		For j=0 To MapHeight-1
			If MapTile(i,j)/100=6 And MapTile(i,j)<603 Then MapTile(i,j)=300
		Next 
	Next
	LevelAddWater(300,WallTextureSide2a,WallTextureSide2a,WaterTexture)
	; (remember: 399 is taken by water with one steel box in it)
	; and restore level data
	For i=0 To MapWidth-1
		For j=0 To MapHeight-1
			MapTile(i,j)=MapTile2(i,j)
		Next 
	Next

	
	; Ice	
	LevelAddFloor(400,IceTexture)
	LevelAddFloor(401,IceCurveTexture1)
	LevelAddFloor(402,IceCurveTexture2)
	LevelAddFloor(403,IceCurveTexture3)
	LevelAddFloor(404,IceCurveTexture4)
	
	; Conveyor Belts
	LevelAddFloor(500,ConveyorTexture1)
	LevelAddFloor(501,ConveyorTexture2)
	LevelAddFloor(502,ConveyorTexture3)
	LevelAddFloor(503,ConveyorTexture4)
	
	; BreakAway Tiles
	; Are not added as tile, but as objects (so each can be removed/changed individually)
	
	; lava, also add second breakaway tiles
	For i=0 To MapWidth-1
		For j=0 To MapHeight-1
			If MapTile(i,j)/100=6 And MapTile(i,j)>=603 Then MapTile(i,j)=900
		Next 
	Next
	LevelAddLava(900,WallTextureSide2a,WallTextureSide2a,LavaTexture)
	; and restore level data
	For i=0 To MapWidth-1
		For j=0 To MapHeight-1
			MapTile(i,j)=MapTile2(i,j)
		Next 
	Next

	; Teleporters
	For i=0 To 3
		LevelAddFloor(1200+i,TeleportTexture(i))
	Next 
	
	; and turn pillars into walls
	For i=0 To NofPillars-1
		MapTile(EntityX(PillarEntity(i))-0.5,-(EntityZ(PillarEntity(i))+0.5))=200
	Next
	
	
	; and finally add background
	; Background effect
	; Use binary addition
	
	If MapBackground=0 Or MapBackground=1 Or MapBackground=3
		; sky, forest, stars
		BackGroundPlane = CreatePlane()
		TranslateEntity BackGroundPlane,0,-50,0
		EntityTexture BackGroundPlane,BackGroundTexture
				
		;CameraClsColor camera,0,192,255
		;CameraFogColor camera,0,192,255
		;CameraFogRange camera,150,200
		;CameraRange camera,.9,200
		;CameraFogMode camera,1
	EndIf
	If MapBackGround=0
		; sky, second layer
		BackGroundPlane2 = CreatePlane()
		EntityTexture BackGroundPlane2, BackGroundTexture
		TranslateEntity BackGroundPlane2,0,-60,0
		EntityAlpha BackGroundPlane,0.5
	EndIf
	
	If MapBackGround = 2
		; Edge Walls (slanted)
		
		BackGroundPlane = CreateMesh()
		
		; left wall
		Brush = CreateBrush()
		If MapStyle=4
			BrushTexture Brush,walltexturetop1
		Else
			BrushTexture Brush,walltextureside2b
		EndIf

		Surface=CreateSurface(BackGroundPlane,Brush)
		AddVertex(Surface,1,0,10,20+MapHeight,14.46)
		AddVertex(Surface,1,0,-10-MapHeight,0,14.46)
		AddVertex(Surface,-9,10,-10-MapHeight,0,0)
		AddVertex(Surface,-9,10,10,20+MapHeight,0)
		AddTriangle(Surface,2,3,0)
		AddTriangle(Surface,2,0,1)
		; back wall
		AddVertex(Surface,-10,10,9,0,0)
		AddVertex(Surface,MapWidth+10,10,9,20+MapWidth,0)
		AddVertex(Surface,-10,0,-1,0,14.46)
		AddVertex(Surface,MapWidth+10,0,-1,20+MapWidth,14.46)
		AddTriangle(Surface,4,5,7)
		AddTriangle(Surface,4,7,6)
		FreeBrush brush
		; right wall
		Brush = CreateBrush()
		If MapStyle=4
			BrushTexture Brush,walltexturetop1
		Else
			BrushTexture Brush,walltextureside2a
		EndIf
		Surface=CreateSurface(BackGroundPlane,Brush)
		AddVertex(Surface,MapWidth+9,10,10,0,0)
		AddVertex(Surface,MapWidth+9,10,-MapHeight-10,20+MapHeight,0)
		AddVertex(Surface,MapWidth-1,0,10,0,14.46)
		AddVertex(Surface,MapWidth-1,0,-MapHeight-10,20+MapHeight,14.46)
		AddTriangle(Surface,0,1,3)
		AddTriangle(Surface,0,3,2)
		; front wall
		AddVertex(Surface,MapWidth+10,10,-MapHeight-9,0,0)
		AddVertex(Surface,-10,10,-MapHeight-9,20+MapWidth,0)
		AddVertex(Surface,MapWidth+10,0,-MapHeight+1,0,14.46)
		AddVertex(Surface,-10,0,-MapHeight+1,20+MapWidth,14.46)
		AddTriangle(Surface,4,5,7)
		AddTriangle(Surface,4,7,6)
		FreeBrush brush
		
		UpdateNormals(BackGroundPlane)
		
	
	EndIf
	
	If MapBackGround = 4
		; Edge Walls (flat)
		
		BackGroundPlane = CreateMesh()
		
		; top wall
		Brush = CreateBrush()
		BrushTexture Brush,walltexturetop1
		Surface=CreateSurface(BackGroundPlane,Brush)
		AddVertex(Surface,-10,.9,10,0,0)
		AddVertex(Surface,MapWidth+10,.9,10,MapWidth+20,0)
		AddVertex(Surface,-10,.9,0,0,10)
		AddVertex(Surface,MapWidth+10,.9,0,MapWidth+20,10)
		AddTriangle(Surface,0,1,2)
		AddTriangle(Surface,1,3,2)
		; left wall
		AddVertex(Surface,-10,.9,0,0,0)
		AddVertex(Surface,0,.9,0,10,0)
		AddVertex(Surface,-10,.9,-MapHeight,0,MapHeight)
		AddVertex(Surface,0,.9,-MapHeight,10,MapHeight)
		AddTriangle(Surface,4,5,6)
		AddTriangle(Surface,5,7,6)
		; right wall
		AddVertex(Surface,MapWidth,.9,0,0,0)
		AddVertex(Surface,MapWidth+10,.9,0,10,0)
		AddVertex(Surface,MapWidth,.9,-MapHeight,0,MapHeight)
		AddVertex(Surface,MapWidth+10,.9,-MapHeight,10,MapHeight)
		AddTriangle(Surface,8,9,10)
		AddTriangle(Surface,9,11,10)
		; bottom wall
		AddVertex(Surface,-10,.9,-MapHeight,0,0)
		AddVertex(Surface,MapWidth+10,.9,-MapHeight,20+MapWidth,0)
		AddVertex(Surface,-10,.9,-MapHeight-10,0,10)
		AddVertex(Surface,MapWidth+10,.9,-MapHeight-10,20+MapWidth,10)
		AddTriangle(Surface,12,13,14)
		AddTriangle(Surface,13,15,14)
		FreeBrush brush
		
		UpdateNormals(BackGroundPlane)
		
	
	EndIf
	If MapBackground=5 
		; water
		BackGroundPlane = CreatePlane()
		TranslateEntity BackGroundPlane,0,-2.001,0
		EntityTexture BackGroundPlane,WallTextureSide2a
		
		BackGroundPlane2 = CreatePlane()		
		TranslateEntity BackGroundPlane2,0,-0.2,0
		EntityTexture BackGroundPlane2,WaterTexture
		EndIf
	If MapBackground=6 
		; lava
		BackGroundPlane = CreatePlane()
		TranslateEntity BackGroundPlane,0,-2.1,0
		EntityTexture BackGroundPlane,WallTextureSide2a
		
		BackGroundPlane2 = CreatePlane()		
		TranslateEntity BackGroundPlane2,0,-0.2,0
		EntityTexture BackGroundPlane2,LavaTexture
	EndIf



	
	
	

		
	
	UpdateNormals LevelMesh
	
	; Add Objects from Tilemap
	For i=0 To MapWidth-1
		For j=0 To MapHeight-1
			; BreakAway Tiles
			If MapTile(i,j)/100=6
				CreateBreakAway(i,j,MapTile(i,j)-600)
			EndIf
		Next 
	Next 
		
		
	
	
	
End Function

; ----------------------------------------------------------
; TILE FUNCTIONS
; These add particular tiletypes, such as flat floors, or 
; water with alpha effects etc. Each one has its own definition
; ----------------------------------------------------------


Function LevelAddFloor(TileType,Tex1)
	; Shape Description
	; Just flat floor, without detail, one texture, at height zero
	


	; First, check if there are any such Tiles
	Flag=0
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)=TileType Then Flag=1
		Next 
	Next
	If Flag=0 Then Return 0  ; no, get out
	
	; Create the Brush
	SurfaceBrush=CreateBrush()
	BrushTexture SurfaceBrush,Tex1
	LevelSurface(NofLevelSurfaces)=CreateSurface(LevelMesh,SurfaceBrush)
	
	ShapeAddFlat(TileType,0)
	
	; And on to the next surface
	NofLevelSurfaces=NofLevelSurfaces+1
	FreeBrush SurfaceBrush
	
	
	
End Function

Function LevelAddWall(TileType,Tex1,Tex2a,Tex2b)
	; Shape Description
	; flat floor at height 1 (tex1), flat walls to the side ascending (tex2), 
	

	; First, check if there are any such Tiles
	Flag=0
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)=TileType Then Flag=1
		Next 
	Next
	If Flag=0 Then Return 0  ; no, get out
	
	; PART 1: Create the Top
	
	; ---------------------------------
	; Create the Brush
	SurfaceBrush=CreateBrush()
	
	BrushTexture SurfaceBrush,Tex1
	LevelSurface(NofLevelSurfaces)=CreateSurface(LevelMesh,SurfaceBrush)
	
	ShapeAddFlat(TileType,.9)
	
	; And on to the next surface
	NofLevelSurfaces=NofLevelSurfaces+1
	FreeBrush SurfaceBrush
		
	
	; PART 2: Create the Light Sides 
	; ---------------------------------
	; Create the Brush
	SurfaceBrush=CreateBrush()
	BrushTexture SurfaceBrush,Tex2a
	LevelSurface(NofLevelSurfaces)=CreateSurface(LevelMesh,SurfaceBrush)
	
	ShapeAddWallsLight(TileType,.9,0)
	
	; And on to the next surface
	NofLevelSurfaces=NofLevelSurfaces+1
	FreeBrush SurfaceBrush
	

	; PART 3: Create the Dark Sides 
	; ---------------------------------
	; Create the Brush
	SurfaceBrush=CreateBrush()
	BrushTexture SurfaceBrush,Tex2b
	LevelSurface(NofLevelSurfaces)=CreateSurface(LevelMesh,SurfaceBrush)
	
	ShapeAddWallsDark(TileType,.9,0)
	
	; And on to the next surface
	NofLevelSurfaces=NofLevelSurfaces+1
	FreeBrush SurfaceBrush
	
	


End Function



Function LevelAddWater(TileType,Tex1,Tex2,Tex3)
	; Shape Description
	; flat floor at height -2 (tex1), flat walls to the side descending (tex2), 
	; extra alpha layer at top (tex3)


	; First, check if there are any such Tiles
	Flag=0
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)=TileType Then Flag=1
		Next 
	Next
	If Flag=0 Then Return 0  ; no, get out
	
	If MapBackGround<>5
		; PART 1: Create the Floor
		; ---------------------------------
		; Create the Brush
		SurfaceBrush=CreateBrush()
		
		BrushTexture SurfaceBrush,Tex1
		LevelSurface(NofLevelSurfaces)=CreateSurface(LevelMesh,SurfaceBrush)
		
		ShapeAddFlat(TileType,-2)
		
		; And on to the next surface
		NofLevelSurfaces=NofLevelSurfaces+1
		FreeBrush SurfaceBrush
	EndIf
	
	
	; PART 2: Create the Sides
	; ---------------------------------
	; Create the Brush
	SurfaceBrush=CreateBrush()
	BrushTexture SurfaceBrush,Tex2
	LevelSurface(NofLevelSurfaces)=CreateSurface(LevelMesh,SurfaceBrush)
	
	ShapeAddWalls(TileType,-2,0)
	
	; And on to the next surface
	NofLevelSurfaces=NofLevelSurfaces+1
	FreeBrush SurfaceBrush
	


	
	If MapBackGround<>5
		; PART 3: Create the Water (Alpha)
		; ---------------------------------
		; Create the Brush
		SurfaceBrush=CreateBrush()
		
		BrushTexture SurfaceBrush,Tex3
		BrushAlpha SurfaceBrush,0.8
		LevelSurface(NofLevelSurfaces)=CreateSurface(LevelMesh,SurfaceBrush)
		
		ShapeAddFlat(TileType,-0.2)
		
		; And on to the next surface
		NofLevelSurfaces=NofLevelSurfaces+1
		FreeBrush SurfaceBrush
	EndIf


End Function

Function LevelAddLava(TileType,Tex1,Tex2,Tex3)
	; Shape Description
	; flat floor at height -2 (tex1), flat walls to the side descending (tex2), 
	; extra alpha layer at top (tex3)


	; First, check if there are any such Tiles
	Flag=0
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)=TileType Then Flag=1
		Next 
	Next
	If Flag=0 Then Return 0  ; no, get out
	
	If MapBackGround<>6
		; PART 1: Create the Floor
		; ---------------------------------
		; Create the Brush
		SurfaceBrush=CreateBrush()
		
		BrushTexture SurfaceBrush,Tex1
		LevelSurface(NofLevelSurfaces)=CreateSurface(LevelMesh,SurfaceBrush)
		
		ShapeAddFlat(TileType,-1)
		
		; And on to the next surface
		NofLevelSurfaces=NofLevelSurfaces+1
		FreeBrush SurfaceBrush
	EndIf
		
	
	; PART 2: Create the Sides
	; ---------------------------------
	; Create the Brush
	SurfaceBrush=CreateBrush()
	BrushTexture SurfaceBrush,Tex2
	LevelSurface(NofLevelSurfaces)=CreateSurface(LevelMesh,SurfaceBrush)
	
	ShapeAddWalls(TileType,-1,0)
	
	; And on to the next surface
	NofLevelSurfaces=NofLevelSurfaces+1
	FreeBrush SurfaceBrush
	


	
	If MapBackGround<>6	
		; PART 3: Create the Water (Alpha)
		; ---------------------------------
		; Create the Brush
		SurfaceBrush=CreateBrush()
		
		BrushTexture SurfaceBrush,Tex3
		BrushAlpha SurfaceBrush,0.8
		LevelSurface(NofLevelSurfaces)=CreateSurface(LevelMesh,SurfaceBrush)
		
		ShapeAddFlat(TileType,-0.2)
		
		; And on to the next surface
		NofLevelSurfaces=NofLevelSurfaces+1
		FreeBrush SurfaceBrush
	EndIf


End Function

Function LevelAddDeepHole(TileType,Tex1a,Tex1b)
	; Shape Description
	; deep deep sidewalls only 

	; First, check if there are any such Tiles
	Flag=0
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)=TileType Then Flag=1
		Next 
	Next
	If Flag=0 Then Return 0  ; no, get out
	
	; PART 1: Create the Light Sides 
	; ---------------------------------
	; Create the Brush
	SurfaceBrush=CreateBrush()
	BrushTexture SurfaceBrush,Tex1a
	LevelSurface(NofLevelSurfaces)=CreateSurface(LevelMesh,SurfaceBrush)
	
	ShapeAddWallsLight(TileType,-50,1)
	
	; And on to the next surface
	NofLevelSurfaces=NofLevelSurfaces+1
	FreeBrush SurfaceBrush
	

	; PART 2: Create the Dark Sides 
	; ---------------------------------
	; Create the Brush
	SurfaceBrush=CreateBrush()
	BrushTexture SurfaceBrush,Tex1b
	LevelSurface(NofLevelSurfaces)=CreateSurface(LevelMesh,SurfaceBrush)
	
	ShapeAddWallsDark(TileType,-50,1)
	
	; And on to the next surface
	NofLevelSurfaces=NofLevelSurfaces+1
	FreeBrush SurfaceBrush

	
	


End Function



; ----------------------------------------------------------
; SHAPE FUNCTIONS
; These are called from within Floor/Water/Wall/etc functions. They only add particular shapes
; (i.e. vertices and triangles) to the mesh with the assigned surface,textures,heights, etc.
; ----------------------------------------------------------


Function ShapeAddFlat(TileType,TileHeight#)
	; just a flat (horizontal) shape 
	
	NofVertices=0
	; start by constructing horizontal surfaces, this is done by going through the maptiles and creating
	; rectangles (as large as possible) to cover all floortiles
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			; i,j is now the new corner to be looked at (topleft)
			; is it a floor?
			If MapTile(i,j)=TileType
				;yes, start expanding to the right
				i2=i
				j2=j
				While i2+1<MapWidth And MapTile(i2+1,j)=TileType
					i2=i2+1
				Wend
				i3=i2 ; furthest right tile (inclusive)
				; now start expanding downwards
				j3=-1
				While j3=-1
					For i2=i To i3
						If j2+1=MapHeight Or MapTile(i2,j2+1)<>TileType
						j3=j2
						EndIf
					Next
					j2=j2+1
				Wend
				; Now i/i3, j/j3 are the corners For the rectangle
				; Add it!
				; Add Vertices (4)
				AddVertex(LevelSurface(NofLevelSurfaces),-.001+i,TileHeight,-j+.001 ,0,0)
				AddVertex(LevelSurface(NofLevelSurfaces),1.001+i3,TileHeight,-j+.001 ,i3-i+1,0)
				AddVertex(LevelSurface(NofLevelSurfaces),-.001+i,TileHeight,-j3-1.001 ,0,j3-j+1)
				AddVertex(LevelSurface(NofLevelSurfaces),1.001+i3,TileHeight,-j3-1.001 ,i3-i+1,j3-j+1)
				; Add Triangles (2)
				AddTriangle(LevelSurface(NofLevelSurfaces),0+NofVertices,1+NofVertices,2+NofVertices) 
				AddTriangle(LevelSurface(NofLevelSurfaces),3+NofVertices,2+NofVertices,1+NofVertices) 
				NofVertices=NofVertices+4
									
				; Finally "delete" those tiles
				For i2=i To i3
					For j2=j To j3
						MapTile(i2,j2)=MapTile(i2,j2)+10000
					Next
				Next
			EndIf
		Next
	Next 
		 	
	
	; Finally go through and undelete
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)>10000 Then MapTile(i,j)=MapTile(i,j)-10000
		Next
	Next

	
End Function 

Function ShapeAddWalls(TileType,TileHeight#,Edge)
	; adds straight walls to the left,right,back and front of the tile

	; edge is set to one if you don't want the first wall to appear at edge of map
	;	(e.g. for tall outside tower walls)
		
	; Step 1: left walls first
	; ------------------------

	NofVertices=0
	For j=0 To MapHeight-1
		For i=Edge To MapWidth-1
			; check if this is a start of this tiletype (i.e. nothing or different to its left)
			If MapTile(i,j)=TileType
				Flag=0
				If i=0
					Flag=1
				Else If MapTile(i-1,j)<>TileType And MapTile(i-1,j)<>TileType+10000
					Flag=1
				EndIf
				If Flag=1
					; now check downward to see how far this condition holds
					j2=j+1
					If i=0 ; none at left (at edge)
						While MapTile(i,j2)=TileType And j2<MapHeight
							j2=j2+1
						Wend
					Else ; different tile at left
						While MapTile(i,j2)=TileType And MapTile(i-1,j2)<>TileType And MapTile(i-1,j2)<>TileType+10000 And j2<MapHeight-1
							j2=j2+1
						Wend
					EndIf
					j2=j2-1
					; Now there is a left wall at i, stretching from j to j2 inclusive
					; Add it
					; Add Vertices (4)
				
					AddVertex(LevelSurface(NofLevelSurfaces),i,TileHeight,-j ,0,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i,0,-j ,0,TileHeight)
					AddVertex(LevelSurface(NofLevelSurfaces),i,TileHeight,-j2-1 ,j2-j+1,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i,0,-j2-1 ,j2-j+1,TileHeight)
					; Add Triangles (2)
					If TileHeight>0
						AddTriangle(LevelSurface(NofLevelSurfaces),0+NofVertices,2+NofVertices,1+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),1+NofVertices,2+NofVertices,3+NofVertices) 
					Else
						AddTriangle(LevelSurface(NofLevelSurfaces),0+NofVertices,3+NofVertices,1+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),0+NofVertices,2+NofVertices,3+NofVertices)
					EndIf
					NofVertices=NofVertices+4
					
					
					; Finally "delete" those tiles
					For j3=j To j2
						MapTile(i,j3)=MapTile(i,j3)+10000
					Next
				EndIf
			EndIf
		Next
	Next
	
	
	; Finally go through and undelete
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)>10000 Then MapTile(i,j)=MapTile(i,j)-10000
		Next
	Next
	
		
	
	; Step 2: right walls 
	; ------------------------
	
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1-Edge
			; check if this is an end of this tiletype (i.e. nothing or different to its right)
			If MapTile(i,j)=TileType
				Flag=0
				If i=MapWidth-1
					Flag=1
				Else If MapTile(i+1,j)<>TileType And MapTile(i+1,j)<>TileType+10000
					Flag=1
				EndIf
				If Flag=1
					; now check downward to see how far this condition holds
					j2=j+1
					If i=MapWidth-1 ; none at right (at edge)
						While MapTile(i,j2)=TileType And j2<MapHeight
							j2=j2+1
						Wend
					Else ; different tile at right
						While MapTile(i,j2)=TileType And MapTile(i+1,j2)<>TileType And MapTile(i+1,j2)<>TileType+10000 And j2<MapHeight-1
							j2=j2+1
						Wend
					EndIf
					j2=j2-1
					; Now there is a left wall at i+1 (since right side), stretching from j to j2 inclusive
					; Add it
					; Add Vertices (4)
					AddVertex(LevelSurface(NofLevelSurfaces),i+1,TileHeight,-j ,0,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i+1,0,-j ,0,TileHeight)
					AddVertex(LevelSurface(NofLevelSurfaces),i+1,TileHeight,-j2-1 ,j2-j+1,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i+1,0,-j2-1 ,j2-j+1,TileHeight)
				
				
					; Add Triangles (2)
					If TileHeight>0
						AddTriangle(LevelSurface(NofLevelSurfaces),0+NofVertices,1+NofVertices,2+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),1+NofVertices,3+NofVertices,2+NofVertices) 
					Else
						AddTriangle(LevelSurface(NofLevelSurfaces),3+NofVertices,0+NofVertices,1+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),2+NofVertices,0+NofVertices,3+NofVertices)
					EndIf
					NofVertices=NofVertices+4
					; Finally "delete" those tiles
					For j3=j To j2
						MapTile(i,j3)=MapTile(i,j3)+10000
					Next
				EndIf
			EndIf
		Next
	Next
	
	
	; Finally go through and undelete
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)>10000 Then MapTile(i,j)=MapTile(i,j)-10000
		Next
	Next

	; Step 3: back walls 
	; ------------------------

	
	For i=0 To MapWidth-1
		For j=Edge To MapHeight-1
			; check if this is a start of this tiletype (i.e. nothing or different to its top)
			If MapTile(i,j)=TileType
				Flag=0
				If j=0
					Flag=1
				Else If MapTile(i,j-1)<>TileType And MapTile(i,j-1)<>TileType+10000
					Flag=1
				EndIf
				If Flag=1
					; now check downward to see how far this condition holds
					i2=i+1
					If j=0 ; none at top (at edge)
						While MapTile(i2,j)=TileType And i2<MapWidth
							i2=i2+1
						Wend
					Else ; different tile at top
						While MapTile(i2,j)=TileType And MapTile(i2,j-1)<>TileType And MapTile(i2,j-1)<>TileType+10000 And i2<MapWidth-1
							i2=i2+1
						Wend
					EndIf
					i2=i2-1
					; Now there is a left wall at j, stretching from i to i2 inclusive
					; Add it
					; Add Vertices (4)
					AddVertex(LevelSurface(NofLevelSurfaces),i,TileHeight,-j ,0,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i,0,-j ,0,TileHeight)
					AddVertex(LevelSurface(NofLevelSurfaces),i2+1,TileHeight,-j ,i2-i+1,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i2+1,0,-j ,i2-i+1,TileHeight)
				
				
					; Add Triangles (2)
					If TileHeight>0
						AddTriangle(LevelSurface(NofLevelSurfaces),1+NofVertices,2+NofVertices,0+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),3+NofVertices,2+NofVertices,1+NofVertices) 
					Else
						AddTriangle(LevelSurface(NofLevelSurfaces),1+NofVertices,3+NofVertices,2+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),1+NofVertices,2+NofVertices,0+NofVertices)
					EndIf
					NofVertices=NofVertices+4
					; Finally "delete" those tiles
					For i3=i To i2
						MapTile(i3,j)=MapTile(i3,j)+10000
					Next
				EndIf
			EndIf
		Next
	Next
	
	
	; Finally go through and undelete
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)>10000 Then MapTile(i,j)=MapTile(i,j)-10000
		Next
	Next
	
	; Step 4: front walls 
	; ------------------------
	
	For i=0 To MapWidth-1
		For j=0 To MapHeight-1-Edge
			; check if this is an end of this tiletype (i.e. nothing or different below)
			If MapTile(i,j)=TileType
				Flag=0
				If j=MapHeight-1
					Flag=1
				Else If MapTile(i,j+1)<>TileType And MapTile(i,j+1)<>TileType+10000
					Flag=1
				EndIf
				If Flag=1
					; now check towards right to see how far this condition holds
					i2=i+1
					If j=MapHeight-1 ; none below (at edge)
						While MapTile(i2,j)=TileType And i2<MapWidth
							i2=i2+1
						Wend
					Else ; different tile below
						While MapTile(i2,j)=TileType And MapTile(i2,j+1)<>TileType And MapTile(i2,j+1)<>TileType+10000 And i2<MapWidth-1
							i2=i2+1
						Wend
					EndIf
					i2=i2-1
					; Now there is a front wall at j+1 (since bottom side), stretching from i to i2 inclusive
					; Add it
					; Add Vertices (4)
					AddVertex(LevelSurface(NofLevelSurfaces),i,TileHeight,-j-1 ,0,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i,0,-j-1 ,0,TileHeight)
					AddVertex(LevelSurface(NofLevelSurfaces),i2+1,TileHeight,-j-1 ,i2-i+1,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i2+1,0,-j-1 ,i2-i+1,TileHeight)
				
				
					; Add Triangles (2)
					If TileHeight>0
						AddTriangle(LevelSurface(NofLevelSurfaces),2+NofVertices,1+NofVertices,0+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),2+NofVertices,3+NofVertices,1+NofVertices) 
					Else
						AddTriangle(LevelSurface(NofLevelSurfaces),1+NofVertices,0+NofVertices,3+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),3+NofVertices,0+NofVertices,2+NofVertices)
					EndIf
					NofVertices=NofVertices+4
					; Finally "delete" those tiles
					For i3=i To i2
						MapTile(i3,j)=MapTile(i3,j)+10000
					Next
				EndIf
			EndIf
		Next
	Next
	
	
	; Finally go through and undelete
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)>10000 Then MapTile(i,j)=MapTile(i,j)-10000
		Next
	Next

	
	


	
	
End Function

Function ShapeAddWallsLight(TileType,TileHeight#,Edge)
	; adds straight walls to the left,right,back and front of the tile

	; edge is set to one if you don't want the first wall to appear at edge of map
	;	(e.g. for tall outside tower walls)
		
	; Step 1: left walls first
	; ------------------------

	NofVertices=0
	For j=0 To MapHeight-1
		For i=Edge To MapWidth-1
			; check if this is a start of this tiletype (i.e. nothing or different to its left)
			If MapTile(i,j)=TileType
				Flag=0
				If i=0
					Flag=1
				Else If MapTile(i-1,j)<>TileType And MapTile(i-1,j)<>TileType+10000
					Flag=1
				EndIf
				If Flag=1
					; now check downward to see how far this condition holds
					j2=j+1
					If i=0 ; none at left (at edge)
						While MapTile(i,j2)=TileType And j2<MapHeight
							j2=j2+1
						Wend
					Else ; different tile at left
						While MapTile(i,j2)=TileType And MapTile(i-1,j2)<>TileType And MapTile(i-1,j2)<>TileType+10000 And j2<MapHeight-1
							j2=j2+1
						Wend
					EndIf
					j2=j2-1
					; Now there is a left wall at i, stretching from j to j2 inclusive
					; Add it
					; Add Vertices (4)
				
					AddVertex(LevelSurface(NofLevelSurfaces),i,TileHeight,-j ,0,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i,0,-j ,0,TileHeight)
					AddVertex(LevelSurface(NofLevelSurfaces),i,TileHeight,-j2-1 ,j2-j+1,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i,0,-j2-1 ,j2-j+1,TileHeight)
					; Add Triangles (2)
					If TileHeight>0
						AddTriangle(LevelSurface(NofLevelSurfaces),0+NofVertices,2+NofVertices,1+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),1+NofVertices,2+NofVertices,3+NofVertices) 
					Else
						AddTriangle(LevelSurface(NofLevelSurfaces),0+NofVertices,3+NofVertices,1+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),0+NofVertices,2+NofVertices,3+NofVertices)
					EndIf
					NofVertices=NofVertices+4
					
					
					; Finally "delete" those tiles
					For j3=j To j2
						MapTile(i,j3)=MapTile(i,j3)+10000
					Next
				EndIf
			EndIf
		Next
	Next
	
	
	; Finally go through and undelete
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)>10000 Then MapTile(i,j)=MapTile(i,j)-10000
		Next
	Next
	
	; Step 3: back walls 
	; ------------------------

	
	For i=0 To MapWidth-1
		For j=Edge To MapHeight-1
			; check if this is a start of this tiletype (i.e. nothing or different to its top)
			If MapTile(i,j)=TileType
				Flag=0
				If j=0
					Flag=1
				Else If MapTile(i,j-1)<>TileType And MapTile(i,j-1)<>TileType+10000
					Flag=1
				EndIf
				If Flag=1
					; now check downward to see how far this condition holds
					i2=i+1
					If j=0 ; none at top (at edge)
						While MapTile(i2,j)=TileType And i2<MapWidth
							i2=i2+1
						Wend
					Else ; different tile at top
						While MapTile(i2,j)=TileType And MapTile(i2,j-1)<>TileType And MapTile(i2,j-1)<>TileType+10000 And i2<MapWidth-1
							i2=i2+1
						Wend
					EndIf
					i2=i2-1
					; Now there is a left wall at j, stretching from i to i2 inclusive
					; Add it
					; Add Vertices (4)
					AddVertex(LevelSurface(NofLevelSurfaces),i,TileHeight,-j ,0,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i,0,-j ,0,TileHeight)
					AddVertex(LevelSurface(NofLevelSurfaces),i2+1,TileHeight,-j ,i2-i+1,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i2+1,0,-j ,i2-i+1,TileHeight)
				
				
					; Add Triangles (2)
					If TileHeight>0
						AddTriangle(LevelSurface(NofLevelSurfaces),1+NofVertices,2+NofVertices,0+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),3+NofVertices,2+NofVertices,1+NofVertices) 
					Else
						AddTriangle(LevelSurface(NofLevelSurfaces),1+NofVertices,3+NofVertices,2+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),1+NofVertices,2+NofVertices,0+NofVertices)
					EndIf
					NofVertices=NofVertices+4
					; Finally "delete" those tiles
					For i3=i To i2
						MapTile(i3,j)=MapTile(i3,j)+10000
					Next
				EndIf
			EndIf
		Next
	Next
	
	
	; Finally go through and undelete
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)>10000 Then MapTile(i,j)=MapTile(i,j)-10000
		Next
	Next

	
	
End Function

Function ShapeAddWallsDark(TileType,TileHeight#,Edge)
	; adds straight walls to the left,right,back and front of the tile

	; edge is set to one if you don't want the first wall to appear at edge of map
	;	(e.g. for tall outside tower walls)

	; Step 2: right walls 
	; ------------------------
	
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1-Edge
			; check if this is an end of this tiletype (i.e. nothing or different to its right)
			If MapTile(i,j)=TileType
				Flag=0
				If i=MapWidth-1
					Flag=1
				Else If MapTile(i+1,j)<>TileType And MapTile(i+1,j)<>TileType+10000
					Flag=1
				EndIf
				If Flag=1
					; now check downward to see how far this condition holds
					j2=j+1
					If i=MapWidth-1 ; none at right (at edge)
						While MapTile(i,j2)=TileType And j2<MapHeight
							j2=j2+1
						Wend
					Else ; different tile at right
						While MapTile(i,j2)=TileType And MapTile(i+1,j2)<>TileType And MapTile(i+1,j2)<>TileType+10000 And j2<MapHeight-1
							j2=j2+1
						Wend
					EndIf
					j2=j2-1
					; Now there is a left wall at i+1 (since right side), stretching from j to j2 inclusive
					; Add it
					; Add Vertices (4)
					AddVertex(LevelSurface(NofLevelSurfaces),i+1,TileHeight,-j ,0,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i+1,0,-j ,0,TileHeight)
					AddVertex(LevelSurface(NofLevelSurfaces),i+1,TileHeight,-j2-1 ,j2-j+1,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i+1,0,-j2-1 ,j2-j+1,TileHeight)
				
				
					; Add Triangles (2)
					If TileHeight>0
						AddTriangle(LevelSurface(NofLevelSurfaces),0+NofVertices,1+NofVertices,2+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),1+NofVertices,3+NofVertices,2+NofVertices) 
					Else
						AddTriangle(LevelSurface(NofLevelSurfaces),3+NofVertices,0+NofVertices,1+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),2+NofVertices,0+NofVertices,3+NofVertices)
					EndIf
					NofVertices=NofVertices+4
					; Finally "delete" those tiles
					For j3=j To j2
						MapTile(i,j3)=MapTile(i,j3)+10000
					Next
				EndIf
			EndIf
		Next
	Next
	
	
	; Finally go through and undelete
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)>10000 Then MapTile(i,j)=MapTile(i,j)-10000
		Next
	Next
	
	; Step 4: front walls 
	; ------------------------
	
	For i=0 To MapWidth-1
		For j=0 To MapHeight-1-Edge
			; check if this is an end of this tiletype (i.e. nothing or different below)
			If MapTile(i,j)=TileType
				Flag=0
				If j=MapHeight-1
					Flag=1
				Else If MapTile(i,j+1)<>TileType And MapTile(i,j+1)<>TileType+10000
					Flag=1
				EndIf
				If Flag=1
					; now check towards right to see how far this condition holds
					i2=i+1
					If j=MapHeight-1 ; none below (at edge)
						While MapTile(i2,j)=TileType And i2<MapWidth
							i2=i2+1
						Wend
					Else ; different tile below
						While MapTile(i2,j)=TileType And MapTile(i2,j+1)<>TileType And MapTile(i2,j+1)<>TileType+10000 And i2<MapWidth-1
							i2=i2+1
						Wend
					EndIf
					i2=i2-1
					; Now there is a front wall at j+1 (since bottom side), stretching from i to i2 inclusive
					; Add it
					; Add Vertices (4)
					AddVertex(LevelSurface(NofLevelSurfaces),i,TileHeight,-j-1 ,0,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i,0,-j-1 ,0,TileHeight)
					AddVertex(LevelSurface(NofLevelSurfaces),i2+1,TileHeight,-j-1 ,i2-i+1,0)
					AddVertex(LevelSurface(NofLevelSurfaces),i2+1,0,-j-1 ,i2-i+1,TileHeight)
				
				
					; Add Triangles (2)
					If TileHeight>0
						AddTriangle(LevelSurface(NofLevelSurfaces),2+NofVertices,1+NofVertices,0+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),2+NofVertices,3+NofVertices,1+NofVertices) 
					Else
						AddTriangle(LevelSurface(NofLevelSurfaces),1+NofVertices,0+NofVertices,3+NofVertices) 
						AddTriangle(LevelSurface(NofLevelSurfaces),3+NofVertices,0+NofVertices,2+NofVertices)
					EndIf
					NofVertices=NofVertices+4
					; Finally "delete" those tiles
					For i3=i To i2
						MapTile(i3,j)=MapTile(i3,j)+10000
					Next
				EndIf
			EndIf
		Next
	Next
	
	
	; Finally go through and undelete
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			If MapTile(i,j)>10000 Then MapTile(i,j)=MapTile(i,j)-10000
		Next
	Next




	


End Function




; ----------------------------------------------------------
; INIDIDUAL MODEL FUNCTIONS
; Simple models such as squares, etc.
; ----------------------------------------------------------

Function CreateFlatSquare(tex)
	
	Mesh=CreateMesh()
	Brush=CreateBrush()
	BrushTexture Brush,tex
	Surface=CreateSurface(Mesh,Brush)

	AddVertex(Surface,0,0,0,0.01,-.99)
	AddVertex(Surface,1,0,0,.99,-.99)
	AddVertex(Surface,1,0,-1,.99,0.01)
	AddVertex(Surface,0,0,-1,0.01,0.01)
	AddTriangle(Surface,0,1,2)
	AddTriangle(Surface,0,2,3)
	
	UpdateNormals(Mesh)
	;HideEntity Mesh

	Return Mesh

End Function


Function CreateElectro()
	; creates 4 meshes for the four different speeds of electros
	
	
	; first do the logic part
	NofElectros=0
	For i=0 To 3
		ElectroExists(i)=False
	Next
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			ElectroMap(i,j)=-1
			If MapTile(i,j)/100=7
				ElectroMap(i,j)=MapTile(i,j) Mod 100			
				; do the type
				ElectroExists(MapTile(i,j) Mod 100)=True
				ElectroTimer(MapTile(i,j) Mod 100)=((MapTile(i,j) Mod 100)+2)*ElectroBaseTimer

			
			EndIf
		Next
	Next	
	
	; now create the meshes

	; four types of electros: 0,1,2,3
	
	For eType=0 To 3
		; First, check if there are any such Tiles
		Flag=0
		For j=0 To MapHeight-1
			For i=0 To MapWidth-1
				If MapTile(i,j)=700+eType Then Flag=1
			Next 
		Next
	
		If Flag=0 Then Goto eNone  ; no, get out
		
		; Create the Mesh
		ElectroEntity(eType)=CreateMesh()
		eSurface=CreateSurface(ElectroEntity(eType))
		TileHeight#=-0.01
		; and add the shape	
		NofVertices=0
		; start by constructing horizontal surfaces, this is done by going through the maptiles and creating
		; rectangles (as large as possible) to cover all floortiles
		For j=0 To MapHeight-1
			For i=0 To MapWidth-1
				; i,j is now the new corner to be looked at (topleft)
				; is it an electro?
				If MapTile(i,j)=700+eType
					;yes, start expanding to the right
					i2=i
					j2=j
					While i2+1<MapWidth And MapTile(i2+1,j)=700+eType
						i2=i2+1
					Wend
					i3=i2 ; furthest right tile (inclusive)
					; now start expanding downwards
					j3=-1
					While j3=-1
						For i2=i To i3
							If j2+1=MapHeight Or MapTile(i2,j2+1)<>700+eType
								j3=j2
							EndIf
						Next
						j2=j2+1
					Wend
					; Now i/i3, j/j3 are the corners For the rectangle
					; Add it!
					; Add Vertices (4)
					AddVertex(eSurface,0+i,TileHeight,-j ,0,0)
					AddVertex(eSurface,1+i3,TileHeight,-j ,i3-i+1,0)
					AddVertex(eSurface,0+i,TileHeight,-j3-1 ,0,j3-j+1)
					AddVertex(eSurface,1+i3,TileHeight,-j3-1 ,i3-i+1,j3-j+1)
					; Add Triangles (2)
					AddTriangle(eSurface,0+NofVertices,1+NofVertices,2+NofVertices) 
					AddTriangle(eSurface,3+NofVertices,2+NofVertices,1+NofVertices) 
					NofVertices=NofVertices+4
									
					; Finally "delete" those tiles
					For i2=i To i3
						For j2=j To j3
							MapTile(i2,j2)=MapTile(i2,j2)+10000
						Next
					Next
				EndIf
			Next
		Next 
		; Finally go through and undelete
		For j=0 To MapHeight-1
			For i=0 To MapWidth-1
				If MapTile(i,j)>10000 Then MapTile(i,j)=MapTile(i,j)-10000
			Next
		Next
		
		EntityTexture ElectroEntity(eType),ElectroTexture,0
		UpdateNormals ElectroEntity(eType)
		; label for skip
		.eNone
	Next
	
	
	
End Function


				
			