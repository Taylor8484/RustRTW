

Function Setup()
	; general Setup routines called at program start

	; Setup Graphics
	
	MyGFXModeWidth(0)=640
	MyGFXModeHeight(0)=480
	MyGFXModeWidth(1)=800
	MyGFXModeHeight(1)=600
	MyGFXModeWidth(2)=1024
	MyGFXModeHeight(2)=768
	MyGFXModeWidth(3)=1280
	MyGFXModeHeight(3)=960

	If GfxMode3DExists(MyGFXModeWidth(MyGfxMode),MyGfxModeHeight(MyGfxMode),8+MyColMode*8)=False
		MyGfxMode=0
		MyColMode=1
		;MyWindowed=0
	EndIf 
	
	
		
	DisplayWidth=MyGFXModeWidth(MyGfxMode)
	DisplayHeight=MyGfxModeHeight(MyGfxMode)
	
	OldGfxMode=MyGfxMode
	OldWindowed=MyWindowed
		

	
	Graphics3D DisplayWidth,DisplayHeight,8+MyColMode*8,MyWindowed*2
	
	SetBuffer BackBuffer()
	Print "========================================="
	Print "Wonderland v "+Str$(VersionNo1)+"."+Str$(VersionNo2)+Str$(VersionNo3)
	If Version<10 Then Print "Shareware/Demo Version"
	Print "========================================="
	Print "Copyright (C) 2002-2004 Midnight Synergy"
	If Language=0
		Print "Visit www.midnightsynergy.com for more information."
	Else
		Print "WWW.MIDNIGHTSYNERGY.COM"
	EndIf
	Print ""
	If Version<10 
		If Lanugage=2
			Print "-----------------------------------------------------------------"
			Print "Si vous avez quelque difficulte a executer ce demo sur votre"
			Print "ordinatuer, referez-vous au fichier 'readme.txt', ou envoyez-nous"
			Print "un courriel a infos@midnightsynergy.com"
			Print "-----------------------------------------------------------------"
		
			Delay 2000
		Else
			Print "----------------------------------------------------------------"
			Print "If you have any difficulties running this demo on your computer"
			Print "try different display settings, refer to the 'readme.txt' file,"
			Print "or email us at infos@midnightsynergy.com"               
			Print "----------------------------------------------------------------"
			Print ""
			Delay 2000
		EndIf
	Else
		Print "-----------------------------------------------------"
		Print "This is the full registered version of Wonderland."
		Print "Please do not pass this copy on to friends or others."
		Print "Report software piracy at infos@midnightsynergy.com"               
		Print "-----------------------------------------------------"
		Print ""

	EndIf
	

	If Language=0
		Print "Loading.... Please Wait"
	Else If Language=1
		Print "Bitte warten..."
	Else If Language=2
		Print ""
		Print "Version Francaise: Dominic Richer, St-Francois du Lac"
		Print ""
		Print "Attendez S.V.P...."
	
	EndIf
	;Flip
	

	InitPackFile()

	; Load Level-Textures
	
	
	
	
	WaterTexture = MyLoadTexture("graphics\water.jpg",2,False)
	

	
	
	
	ConveyorTexture1 = MyLoadTexture ("res8.dat",1,False)
	ConveyorTexture2 = MyLoadTexture ("res8.dat",1,False)
	RotateTexture ConveyorTexture2,270
	ConveyorTexture3 = MyLoadTexture ("res8.dat",1,False)
	RotateTexture ConveyorTexture3,180
	ConveyorTexture4 = MyLoadTexture ("res8.dat",1,False)
	RotateTexture ConveyorTexture4,90
	
	If (Version>=10)
		IceTexture = MyLoadTexture ("graphics\ice.jpg",1,False)
		IceCurveTexture1 = MyLoadTexture ("graphics\icecurve1.jpg",1,False)
		IceCurveTexture2 = MyLoadTexture ("graphics\icecurve2.jpg",1,False)
		IceCurveTexture3 = MyLoadTexture ("graphics\icecurve3.jpg",1,False)
		IceCurveTexture4 = MyLoadTexture ("graphics\icecurve4.jpg",1,False)
		LavaTexture=MyLoadTexture ("graphics\lava.jpg",1,False)
	EndIf
	Print "."	
	
	ElectroTexture=MyLoadAnimTexture("res11.dat",6,64,64,0,4,False)

	
	
	

	
		
		
	; Object Textures
	
	RainbowTexture=MyLoadTexture("graphics\rainbow.jpg",2,False)
	
	
		
	; Load Models
	StinkyMesh = MyLoadMD2("graphics\stinky.md2",False)
	StinkyTexture = MyLoadTexture ("graphics\stinky2.bmp",1,False)
	StinkyTextureB = MyLoadTexture ("graphics\stinky2b.bmp",1,False)
	StinkyTextureC = MyLoadTexture ("graphics\stinky2c.bmp",1,False)
	StinkyTextureD = MyLoadTexture ("graphics\stinky2d.bmp",1,False)
	HideEntity StinkyMesh
	
	LoofMesh = MyLoadMD2("graphics\stinky.md2",False)
	LoofTexture = MyLoadTexture ("graphics\loof.bmp",1,False)
	LoofTextureB = MyLoadTexture ("graphics\loofb.bmp",1,False)
	LoofTextureC = MyLoadTexture ("graphics\loofc.bmp",1,False)
	LoofTextureD = MyLoadTexture ("graphics\loofd.bmp",1,False)
	EntityTexture LoofMesh,LoofTexture
	ScaleEntity LoofMesh,.9,0.7,.95
	HideEntity LoofMesh
	Print "."
	
	WoodBoxMesh = MyLoadMesh("graphics\box.3ds",False)
	WoodBoxTexture = MyLoadTexture ("graphics\woodbox.bmp",1,False)
	EntityTexture WoodBoxMesh,WoodBoxTexture
	ScaleEntity WoodBoxMesh,1.1,1.1,1.1
	HideEntity WoodBoxMesh
;	BoxShadowTexture=LoadTexture("graphics\box\boxshadow.bmp",5)
;	BoxShadowMesh=CreateFlatSquare(BoxShadowTexture)
;	EntityAlpha BoxShadowMesh,0.5	
;	HideEntity BoxShadowMesh


	

	SteelBoxMesh = MyLoadMesh("graphics\box.3ds",False)
	SteelBoxTexture = MyLoadTexture ("graphics\steelbox.bmp",1,False)
	EntityTexture SteelBoxMesh,SteelBoxTexture
;	ScaleEntity SteelBoxMesh,.4,.4,.4
	HideEntity SteelBoxMesh

		
	CannonMesh = MyLoadMesh("graphics\scouge.3ds",False)
	CannonTexture = MyLoadTexture ("graphics\scouge3.bmp",1,False)
	;PositionMesh CannonMesh,0,0,-0.36
;	RotateEntity CannonMesh,0,0,90
	RotateEntity CannonMesh,-90,-90,0
	EntityTexture CannonMesh,CannonTexture
	HideEntity CannonMesh
	Print "."	
	FireBallMesh = CreateSphere()
	EntityColor FireBallMesh,255,240,200
	EntityAlpha FireBallMesh,0.5
	;EntityShininess  FireBallMesh,1
	ScaleEntity FireballMesh,0.2,0.2,0.2
	HideEntity FireBallMesh
	
	RoundShadowTexture = MyLoadTexture ("graphics\beachballshadow.bmp",2,False)
	;RoundShadowMesh = CreateFlatSquare(RoundShadowTexture)
	;HideEntity RoundShadowMesh
	
	IronGateMesh = MyLoadMesh("graphics\gate.3ds",False)
	PositionMesh IronGateMesh,0,0.1,0
	TurnEntity IronGateMesh,-90,0,0
	ScaleEntity IronGateMesh,1,1,1.3
	HideEntity IronGateMesh
	
	MonsterMesh(0) = MyLoadMD2("graphics\coily.md2",False);CreateSphere()
	MonsterTexture(0)=MyLoadTexture ("graphics\coily.bmp",1,False) 
;	RotateMesh MonsterMesh(0),0,0,-90
	ScaleEntity MonsterMesh(0),0.3,0.2,0.3
	;RotateEntity MonsterMesh(0),0,90,0
	;EntityColor MonsterMesh(0),160,160,160
	EntityTexture MonsterMesh(0),MonsterTexture(0)
	HideEntity MonsterMesh(0)
	;MonsterShadowMesh(0) = CreateFlatSquare(RoundShadowTexture)
	;ScaleMesh MonsterShadowMesh(0),.8,1,.8
	;HideEntity MonsterShadowMesh(0)
	Print "."
	
	MonsterMesh(1) = MyLoadMesh("graphics\zbot.3ds",False)	
	MonsterTexture(1)=MyLoadTexture ("graphics\zbot.bmp",1,False) 
	ScaleEntity MonsterMesh(1),0.6,0.6,0.6
	EntityTexture MonsterMesh(1),MonsterTexture(1)
	HideEntity MonsterMesh(1)
	
;	MonsterShadowMesh(1) = CreateFlatSquare(BeachBallShadowTexture)
;	ScaleMesh MonsterShadowMesh(1),.8,1,.8
;	HideEntity MonsterShadowMesh(1)

	
	MonsterMesh(3) = MyLoadMD2("graphics\bomb.md2",False)
	MonsterTexture(3)=MyLoadTexture ("graphics\bomb.bmp",1,False) 
	RotateEntity MonsterMesh(3),0,180,0
	
	;ScaleEntity MonsterMesh(3),0.4,0.4,0.4
	EntityTexture MonsterMesh(3),MonsterTexture(3)
	HideEntity MonsterMesh(3)
;	MonsterShadowMesh(3) = CreateFlatSquare(BeachBallShadowTexture)
;	ScaleMesh MonsterShadowMesh(3),.8,1,.8
;	HideEntity MonsterShadowMesh(3)


	FishMesh= MyLoadMesh("graphics\fish.x",False)
	FitMesh FishMesh,-.5,-.5,-.5,.5,.5,.5,False
	FishTexture = MyLoadTexture("graphics\fish.bmp",1,False)
	EntityTexture FishMesh,FishTexture
	HideEntity FishMesh	
	Print "."
	; Particles
	; just two types: sprite (0) and flatsquare (1)
	ParticleMesh(0)=CreateSprite()
	ScaleEntity ParticleMesh(0),1,1,0.001
	ParticleMesh(1)=CreateFlatSquare(WaterTexture) ;placeholder texture
	HideEntity ParticleMesh(0)
	HideEntity ParticleMesh(1)
	; and Textures
	ParticleTexture=MyLoadAnimTexture ("res18.dat",2,32,32,0,12,False)
	ParticleTexture2=MyLoadAnimTexture ("res19.dat",4,32,32,0,2,False)
	; 0-cannonball, 100-woodsplinter, 101-steelsplinter, 3-rainbow star, 4-water ripple
	; 5,6,7,8 - colour flares, 9-"250"

	


	

		
	; Construct Models
	BreakAwayMesh=CreateFlatSquare(RainbowTexture) ; for now, since its textured later
	HideEntity BreakAwayMesh
	
	ElectroMesh=CreateFlatSquare(ElectroTexture)
	HideEntity ElectroMesh

	Print "."
	;crystal
	ItemMesh(0)=MyLoadMesh("graphics\coin.3ds",False)
	ScaleEntity ItemMesh(0),.8,.8,0.5
	ItemTexture(0)=MyLoadTexture("graphics\coin2.bmp",2,False)
	ItemShadowTexture(0)=MyLoadTexture("graphics\coinshadow.bmp",2,False)
	ItemShadowMesh(0)=CreateFlatSquare(ItemShadowTexture(0))
	PositionMesh ItemShadowMesh(0),-0.5,0,0.5
	ScaleEntity ItemShadowMesh(0),.6,.6,0.1
	EntityAlpha ItemShadowMesh(0),.5
	HideEntity ItemShadowMesh(0)
		
	EntityTexture ItemMesh(0),ItemTexture(0)
	HideEntity ItemMesh(0)
	;endgate
	ItemMesh(1)=CreateSphere()
	ScaleEntity ItemMesh(1),0.5,0.5,0.5
	EntityTexture ItemMesh(1),RainbowTexture
	HideEntity ItemMesh(1)
	;time
	ItemMesh(2)=CreateSphere()
	ScaleEntity ItemMesh(2),.2,.2,.2
	HideEntity ItemMesh(2)
	;money
	ItemMesh(3)=MyLoadMesh("graphics\coin.3ds",False)
	ScaleEntity ItemMesh(3),.8,.8,.5
	ItemTexture(3)=MyLoadTexture("graphics\coin.jpg",1,False)
	EntityTexture ItemMesh(3),ItemTexture(3)
	HideEntity ItemMesh(3)
	Print "."

	ButtonOnTexture=MyLoadTexture("graphics\button.bmp",4,False)
	ButtonOffTexture=MyLoadTexture("graphics\buttonoff.bmp",4,False)

	
	
	PillarMesh(0)=MyLoadMesh("graphics\pillar.3ds",False)
	PillarTexture(0)=MyLoadTexture("graphics\pillar1.jpg",1,False)
	EntityTexture PillarMesh(0),PillarTexture(0)
	RotateEntity PillarMesh(0),-90,0,0
	HideEntity PillarMesh(0)
	PillarMesh(1)=MyLoadMesh("graphics\pyramid.3ds",False)
	PillarTexture(1)=MyLoadTexture("graphics\pillar2.jpg",1,False)
	EntityTexture PillarMesh(1),PillarTexture(1)
	RotateEntity PillarMesh(1),-90,0,0
	HideEntity PillarMesh(1)
	PillarMesh(2)=MyLoadMesh("graphics\spike.3ds",False)
	PillarTexture(2)=MyLoadTexture("graphics\pillar2.jpg",1,False)
	EntityTexture PillarMesh(2),PillarTexture(2)
	RotateEntity PillarMesh(2),-90,0,0
	HideEntity PillarMesh(2)

	;PositionEntity PillarMesh,3.5,0,-3.5
	
	
	; And other stuff
	
	
	DemoTexture(0)=MyLoadTexture("graphics\demo1.jpg",4,False)
	DemoTexture(1)=MyLoadTexture("graphics\demo3.jpg",4,False)
	DemoTexture(2)=MyLoadTexture("graphics\demo4.jpg",4,False)

	
		
	;fntArial=LoadFont("Arial",18,False,False,False)
	;SetFont fntArial
	
	; Setup Camera
	Camera=CreateCamera()
	RotateEntity Camera,0,0,0
	PositionEntity Camera,0,0,-4
	;CameraFogColor Camera,0,0,0
	CameraRange Camera,.98,140
	;CameraFogRange Camera,.98,25
	;CameraFogMode Camera,1
	CameraZoom Camera,2
	
	; Setup Lights
	LevelLight=CreateLight(1)
	LightColor LevelLight,255,0,255
	PositionEntity LevelLight,100,100,-100
	RotateEntity LevelLight,25,-25,0
	Print "."

	; Fonts and Signs
	SetupFont()


	
	
	
	
	

	Print "."
	LoadSounds()	
	Print "."

	MenuSetup()	
	
	
End Function


Function LoadStyleTextures()

	; Note that all textures have to be assigned, since they will all be freed at the end
	
	;MapStyle =Rand(0,3)	
	If MapStyle=0
		; Style 0 - Cave
		

		FloorTexture = MyLoadTexture ("graphics\cavefloor1.jpg",1,False)
	
		WallTextureTop1 = MyLoadTexture ("graphics\cavewalltop1.jpg",1,False)
		FloorTextureSign = MyLoadTexture ("graphics\cavefloorsign.jpg",1,False)

		
		WallTextureSide1a = MyLoadTexture ("graphics\cavewallside1a.jpg",1,False)
		WallTextureSide1b = MyLoadTexture ("graphics\cavewallside1b.jpg",1,False)
		WallTextureSide2a = MyLoadTexture ("graphics\cavewallside2a.jpg",1,False)
		WallTextureSide2b = MyLoadTexture ("graphics\cavewallside2b.jpg",1,False)
		
		BreakAwayTexture1=MyLoadTexture ("graphics\cavebreak1.jpg",1,False)
		BreakAwayTexture2=MyLoadTexture ("graphics\cavebreak2.jpg",1,False)
		BreakAwayTexture3=MyLoadTexture ("graphics\cavebreak3.jpg",1,False)
		
		TeleportTexture(0)=MyLoadTexture ("graphics\cavetele0.jpg",1,False)
		TeleportTexture(1)=MyLoadTexture ("graphics\cavetele1.jpg",1,False)
		TeleportTexture(2)=MyLoadTexture ("graphics\cavetele2.jpg",1,False)
		TeleportTexture(3)=MyLoadTexture ("graphics\cavetele3.jpg",1,False)


		
		
	EndIf
	
	If MapStyle=1
		; Style 1 - Sand
		FloorTexture = MyLoadTexture ("graphics\sandfloor1.jpg",1,False)
		FloorTextureSign = MyLoadTexture ("graphics\sandfloorsign.jpg",1,False)

	
		WallTextureTop1 = MyLoadTexture ("graphics\sandwalltop1.jpg",1,False)
		
		WallTextureSide1a = MyLoadTexture ("graphics\sandwallside1a.jpg",1,False)
		WallTextureSide1b = MyLoadTexture ("graphics\sandwallside1b.jpg",1,False)
		WallTextureSide2a = MyLoadTexture ("graphics\sandwallside2a.jpg",1,False)
		WallTextureSide2b = MyLoadTexture ("graphics\sandwallside2b.jpg",1,False)
		
		BreakAwayTexture1=MyLoadTexture ("graphics\sandbreak1.jpg",1,False)
		BreakAwayTexture2=MyLoadTexture ("graphics\sandbreak2.jpg",1,False)
		BreakAwayTexture3=MyLoadTexture ("graphics\sandbreak3.jpg",1,False)
		
		TeleportTexture(0)=MyLoadTexture ("graphics\sandtele0.jpg",1,False)
		TeleportTexture(1)=MyLoadTexture ("graphics\sandtele1.jpg",1,False)
		TeleportTexture(2)=MyLoadTexture ("graphics\sandtele2.jpg",1,False)
		TeleportTexture(3)=MyLoadTexture ("graphics\sandtele3.jpg",1,False)


		
		
	EndIf
	
	If MapStyle=2
		; Style 2 - Wood
		FloorTexture = MyLoadTexture ("graphics\woodfloor1.jpg",1,False)
		FloorTextureSign = MyLoadTexture ("Graphics\woodfloorsign.jpg",1,False)

	
		WallTextureTop1 = MyLoadTexture ("graphics\woodwalltop1.jpg",1,False)
		
		WallTextureSide1a = MyLoadTexture ("graphics\woodwallside1a.jpg",1,False)
		WallTextureSide1b = MyLoadTexture ("graphics\woodwallside1b.jpg",1,False)
		WallTextureSide2a = MyLoadTexture ("graphics\woodwallside2a.jpg",1,False)
		WallTextureSide2b =MyLoadTexture ("graphics\woodwallside2b.jpg",1,False)
		
		BreakAwayTexture1=MyLoadTexture ("graphics\woodbreak1.jpg",1,False)
		BreakAwayTexture2=MyLoadTexture ("graphics\woodbreak2.jpg",1,False)
		BreakAwayTexture3=MyLoadTexture ("graphics\woodbreak3.jpg",1,False)
		
		TeleportTexture(0)=MyLoadTexture ("graphics\woodtele0.jpg",1,False)
		TeleportTexture(1)=MyLoadTexture ("graphics\woodtele1.jpg",1,False)
		TeleportTexture(2)=MyLoadTexture ("graphics\woodtele2.jpg",1,False)
		TeleportTexture(3)=MyLoadTexture ("graphics\woodtele3.jpg",1,False)



		
		
	EndIf

	If MapStyle=3
		; Style 3 - Metal
		FloorTexture = MyLoadTexture ("graphics\metalfloor1.jpg",1,False)
		FloorTextureSign = MyLoadTexture ("graphics\metalfloorsign.jpg",1,False)
	
		WallTextureTop1 = MyLoadTexture ("graphics\metalwalltop1.jpg",1,False)
		
		WallTextureSide1a = MyLoadTexture ("graphics\metalwallside1a.jpg",1,False)
		WallTextureSide1b = MyLoadTexture ("graphics\metalwallside1b.jpg",1,False)
		WallTextureSide2a = MyLoadTexture ("graphics\metalwallside1a.jpg",1,False)
		WallTextureSide2b = MyLoadTexture ("graphics\metalwallside1b.jpg",1,False)
		
		BreakAwayTexture1=MyLoadTexture ("graphics\metalbreak1.jpg",1,False)
		BreakAwayTexture2=MyLoadTexture ("graphics\metalbreak2.jpg",1,False)
		BreakAwayTexture3=MyLoadTexture ("graphics\metalbreak3.jpg",1,False)
		
		TeleportTexture(0)=MyLoadTexture ("graphics\metaltele0.jpg",1,False)
		TeleportTexture(1)=MyLoadTexture ("graphics\metaltele1.jpg",1,False)
		TeleportTexture(2)=MyLoadTexture ("graphics\metaltele2.jpg",1,False)
		TeleportTexture(3)=MyLoadTexture ("graphics\metaltele3.jpg",1,False)


		
		

		
		
	EndIf
	
	If MapStyle=4
		; Style 4 - Castle
		FloorTexture = MyLoadTexture ("graphics\castlefloor1.jpg",1,False)
	
		WallTextureTop1 = MyLoadTexture ("graphics\castlewalltop1.jpg",1,False)
		FloorTextureSign = MyLoadTexture ("graphics\castlefloorsign.jpg",1,False)


		
		WallTextureSide1a = MyLoadTexture ("graphics\castlewallside1a.jpg",1,False)
		WallTextureSide1b = MyLoadTexture ("graphics\castlewallside1b.jpg",1,False)
		WallTextureSide2a = MyLoadTexture ("res9.dat",1,False)
		WallTextureSide2b = MyLoadTexture ("res10.dat",1,False)
		
		BreakAwayTexture1=MyLoadTexture ("graphics\castlebreak1.jpg",1,False)
		BreakAwayTexture2=MyLoadTexture ("graphics\castlebreak1.jpg",1,False)
		BreakAwayTexture3=MyLoadTexture ("graphics\castlebreak1.jpg",1,False)
		
		TeleportTexture(0)=MyLoadTexture ("graphics\castletele0.jpg",1,False)
		TeleportTexture(1)=MyLoadTexture ("graphics\castletele1.jpg",1,False)
		TeleportTexture(2)=MyLoadTexture ("graphics\castletele2.jpg",1,False)
		TeleportTexture(3)=MyLoadTexture ("graphics\castletele3.jpg",1,False)

	EndIf
	
	If MapStyle=5
		; Style 5 - Jade
		FloorTexture = MyLoadTexture ("graphics\greenfloor1.jpg",1,False)
	
		WallTextureTop1 = MyLoadTexture ("graphics\greenwalltop1.jpg",1,False)
		FloorTextureSign = MyLoadTexture ("graphics\greenfloorsign.jpg",1,False)


		
		WallTextureSide1a = MyLoadTexture ("graphics\greenwallside1a.jpg",1,False)
		WallTextureSide1b = MyLoadTexture ("graphics\greenwallside1b.jpg",1,False)
		WallTextureSide2a = MyLoadTexture ("graphics\greenwallside1a.jpg",1,False)
		WallTextureSide2b = MyLoadTexture ("graphics\greenwallside1b.jpg",1,False)
		
		BreakAwayTexture1=MyLoadTexture ("graphics\greenbreak1.jpg",1,False)
		BreakAwayTexture2=MyLoadTexture ("graphics\greenbreak2.jpg",1,False)
		BreakAwayTexture3=MyLoadTexture ("graphics\greenbreak3.jpg",1,False)
		
		TeleportTexture(0)=MyLoadTexture ("graphics\greentele0.jpg",1,False)
		TeleportTexture(1)=MyLoadTexture ("graphics\greentele1.jpg",1,False)
		TeleportTexture(2)=MyLoadTexture ("graphics\greentele2.jpg",1,False)
		TeleportTexture(3)=MyLoadTexture ("graphics\greentele3.jpg",1,False)

	EndIf


	;MapBackground=6;Rand(0,3)		
	; Background
	If MapBackground=0
		;sky
		BackGroundTexture=MyLoadTexture ("res20.dat",1,False)
		ScaleTexture BackGroundTexture,50,50
	EndIf
	If MapBackground=1
		;forest
		BackGroundTexture=MyLoadTexture ("graphics\grass4.jpg",1,False)
		ScaleTexture BackGroundTexture,50,50
	EndIf
	If MapBackground=3
		;stars
		BackGroundTexture=MyLoadTexture ("graphics\stars.jpg",1,False)
		ScaleTexture BackGroundTexture,50,50
	EndIf
	
	If MapBackground=5 Or MapBackGround=6
		;water/lava
	
	EndIf


	
	
	



End Function