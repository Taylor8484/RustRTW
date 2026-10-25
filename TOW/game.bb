AppTitle "Wonderland"

Include "globals.bb"


LoadOptions()
Setup()



If Version<10 Or Version=11
	StartMenu(0)
Else
	StartMenu(1)
EndIf

;PlayMusic("d:\3dstinky\dream.mp3")



; the Main Loop

WasWindowactive=True

While GameExit=False

  If GetActiveWindow()
	
	If GamePhase=0
		; menu
		If WasWindowActive=False
			If DoesSoundExist=True Then	ChannelVolume Music,OptionsMusicLevel
			WasWindowActive=True
		EndIf

		MenuLoop()
		
	EndIf
	If GamePhase=1
		If WasWindowActive=False
			If DoesSoundExist=True Then	ChannelVolume Music,OptionsMusicLevel
			WasWindowActive=True
		EndIf

		; main game
		MainGameLoop()
		
		; main game
		If KeyDown(1)=True
			PlayMySound(SFXMenuSound(0),0)
			Repeat
			Until KeyDown(1)=False

			StartMenu(5)
		EndIf
		
		

	EndIf
	
  Else
	If WasWindowActive=True
		If DoesSoundExist=True Then	ChannelVolume Music,0
		WasWindowActive=False
	EndIf
	Delay 100
  EndIf

	
		
Wend



; and get outta here
; delete .tmp files
dir=ReadDir(CurrentDir$())
Repeat
	a$=NextFile$(dir)
	
	If Lower$(Right$(a$,4))=".tmp" Or Lower$(Right$(a$,4))=".3ds" Or Lower$(Right$(a$,2))=".x"
		
		If packcleanup=True Then DeleteFile a$
	EndIf
	
Until a$=""
CloseDir dir

ClearWorld
EndGraphics 
;PakClean()

If DoesSoundExist=True
;	If ChannelPlaying(Music)=True
	For i=1 To 100
		ChannelVolume Music,1-Float(i)/100.0
		Delay 10
	Next 
	StopChannel(Music)
;	EndIf
EndIf


If VersionNo1=-999
	ExecFile("http://www.midnightsynergy.com/wonderland/hints")
	End
EndIf

If VersionNo1=-998
	ExecFile("http://www.midnightsynergy.com")
	End
EndIf


If VersionNo1=-1

	ExecFile("http://www.midnightsynergy.com/wonderland/order1.htm")
EndIf
If VersionNo1=-2
	ExecFile("http://www.midnightsynergy.com/games.htm")
EndIf

If VersionNo1=-3
	ExecFile("mailto:bonus@midnightsynergy.com")
EndIf

End







Function StartLevel(Level)

	


	; Load Models
	FreeEntity StinkyMesh
	FreeTexture StinkyTexture
	FreeTexture StinkyTextureB
	FreeTexture StinkyTextureC
	FreeTexture StinkyTextureD
	
	StinkyMesh = MyLoadMD2("graphics\stinky.md2",False)
	StinkyTexture = MyLoadTexture ("graphics\stinky2.bmp",1,False)
	StinkyTextureB = MyLoadTexture ("graphics\stinky2b.bmp",1,False)
	StinkyTextureC = MyLoadTexture ("graphics\stinky2c.bmp",1,False)
	StinkyTextureD = MyLoadTexture ("graphics\stinky2d.bmp",1,False)


	EntityTexture StinkyMesh,StinkyTexture
	
	HideEntity StinkyMesh



	; Free Fonts
	For l.LetterType=Each LetterType
		FreeEntity l\Entity
		Delete l
	Next

	
	RotateEntity Camera,90,0,0
	LightColor LevelLight,-255,-255,-255
	AmbientLight 0,0,0
	GamePhase=1

	LoofExists=False 
	LoofMoving=False
	LoofDead=False
	LoofDying=False
	LoofDyingTimer=0	

	FreeCameraX=0
	FreeCameraZ=0
	
	
	
	LoadLevel(WorldLevelName$(Level))
		
	; make the level mesh
	ConstructLevel()
	
	; and other stuff
	FPScounter=0
	FPStime=MilliSecs()
	FPS=0
	
	TweenPeriod=1000/85
	TweenTime=MilliSecs()-TweenPeriod
	
	LevelCompleted=0
	LevelCompletedTimer=0
	LevelTimer=0
	
	CurrentLevelScore=0
	
	MapMilliTime=MilliSecs()
	

	StartSign(MapName$,16,10,0)
	
	FinalGateAnim=0
	StinkyShrink=0
	LoofShrink=0
	
	ShiftKeyPressed=0
	
	

	
	
	
	
	
End Function

Function EndLevel()

	
		
		

	; clean up EVERYTHING
	
	; Level
	FreeEntity LevelMesh
	NofLevelSurfaces=0
	
	; Objects
	FreeEntity StinkyEntity
	If StinkyDying=False Then FreeEntity StinkyShadowEntity
	
	If LoofExists=True
		FreeEntity LoofEntity
		If LoofDying=False Then FreeEntity LoofShadowEntity
	EndIf
	
	For i=0 To NofBoxes-1
		FreeEntity BoxEntity(i)
	Next
	NofBoxes=0
	
	For i=0 To NofBreakAways-1
		FreeEntity BreakAwayEntity(i)
	Next
	NofBreakAways=0

	For i=0 To 3
		If ElectroExists(i)=True Then FreeEntity ElectroEntity(i)
	Next
	NofElectros=0


	For i=0 To NofCannons-1
		FreeEntity CannonEntity(i)
	Next
	NofCannons=0


	For i=0 To NofFireballs-1
		FreeEntity FireballEntity(i)
	Next
	NofFireballs=0

	For i=0 To NofLaserGates-1
		FreeEntity LaserGateEntity(i)
		FreeEntity LaserGateEntity2(i)
	Next	
	NofLasergates=0
	
	For i=0 To NofButtons-1
		FreeEntity ButtonEntity(i)
	Next
	NofButtons=0

	For i=0 To NofMonsters-1
		FreeEntity MonsterEntity(i)
		If MonsterDying(i)=False Then FreeEntity MonsterShadowEntity(i)
	Next
	NofMonsters=0
	
	For i=0 To NofFish-1
		FreeEntity FishEntity(i)
	Next
	NofFish=0
	
	For i=0 To NofPillars-1
		FreeEntity PillarEntity(i)
	Next
	NofPillars=0


	For i=0 To NofItems-1
		FreeEntity ItemEntity(i)
		If ItemType(i)=0 Or ItemType(i)=1 Or ItemType(i)=3
			FreeEntity ItemShadowEntity(i)
		EndIf
	Next
	NofItems=0
	NofCrystals=0
	
	For p.particle= Each particle
		FreeEntity p\Entity
		Delete p
	Next
	NofParticles=0
	
	
	; Style Textures
	FreeTexture FloorTexture 
	FreeTexture FloorTextureSign
	FreeTexture WallTextureTop1
	FreeTexture WallTextureSide1a
	FreeTexture WallTextureSide1b
	FreeTexture WallTextureSide2a
	FreeTexture WallTextureSide2b
	FreeTexture BreakAwayTexture1
	FreeTexture BreakAwayTexture2
	FreeTexture BreakAwayTexture3
	FreeTexture TeleportTexture(0)
	FreeTexture TeleportTexture(1)
	FreeTexture TeleportTexture(2)
	FreeTexture TeleportTexture(3)


	
	; Background effect
	If MapBackground=0 
		FreeEntity BackGroundPlane
		FreeEntity BackGroundPlane2
		FreeTexture BackGroundTexture
	EndIf
	If MapBackground=1
		FreeEntity BackGroundPlane
		FreeTexture BackGroundTexture
	EndIf
	If MapBackground=2 Or MapBackGround=4
		FreeEntity BackGroundPlane
	EndIf
	If MapBackground=3
		FreeEntity BackGroundPlane
		FreeTexture BackGroundTexture
	EndIf
	If MapBackground=5 Or MapBackGround=6
		FreeEntity BackGroundPlane
		FreeEntity BackGroundPlane2
	EndIf



	
	
	
	
	; 4, 8, 16, etc.
	
	
	; Fonts
	For l.LetterType=Each LetterType
		FreeEntity l\Entity
		Delete l
	Next
	
	
	

End Function






Function MainGameLoop()
	Repeat
		TweenElapsed=MilliSecs()-TweenTime
	Until TweenElapsed>TweenPeriod
	
	If TweenElapsed>20*TweenPeriod 
		TweenElapsed=20*TweenPeriod
		TweenTime=MilliSecs()-TweenElapsed
	EndIf

	;how many 'frames' have elapsed	
	TweenTicks=TweenElapsed/TweenPeriod
	
	
	
	;fractional remainder
	Tween#=Float(TweenElapsed Mod TweenPeriod)/Float(TweenPeriod)

	
	
	
	

	
	
	
	For k=1 To TweenTicks
		Tweentime=Tweentime+Tweenperiod
		If k=Tweenticks 
			CaptureWorld
			
		EndIf

		
		; do the game in here
		UpdateGame()		
		
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	


	

	; fps calculation
	fpscounter=fpscounter+1
	If fpscounter=100
		fpscounter=0
		fps=(MilliSecs()-fpstime)
		fpstime=MilliSecs()
		fps=100000/(fps)
	
	EndIf

	
	; Data Display
	
;	DisplayText(Str$(FreeCameraX)+" "+Str$(MonsterY(1))+" "+Str$(MonsterX2(1))+" "+Str$(MonsterY2(1))+" "+Str$(BoxX(0))+" "+Str$(BoxY(0)),0,3,1,9)	
	; Crystals
	If NofCrystals>0 Or LevelTimer Mod 40<20
		;If NofCrystals>=10
		If Language=0
			DisplayText("Coins: "+Str$(NofTotalCrystals-NofCrystals),0,0,1,0)
		Else If Language=1
			DisplayText("Taler: "+Str$(NofTotalCrystals-NofCrystals),0,0,1,0)
		Else If Language=2
			DisplayText("Sous:  "+Str$(NofTotalCrystals-NofCrystals),0,0,1,0)

		EndIf
		;Else 
	;		DisplayText("Coins: 0"+Str$(NofTotalCrystals-NofCrystals),0,0,1,0)
		;EndIf
		If Language=0
			If NofTotalCrystals-NofCrystals>=10
				DisplayText(" of "+Str$(NofTotalCrystals),9,0,1,5)
			Else 
				DisplayText(" of "+Str$(NofTotalCrystals),8,0,1,5)
			EndIf
		Else If Language=1
			If NofTotalCrystals-NofCrystals>=10
				DisplayText(" von "+Str$(NofTotalCrystals),9,0,1,5)
			Else 
				DisplayText(" von "+Str$(NofTotalCrystals),8,0,1,5)
			EndIf
		Else If Language=2
			If NofTotalCrystals-NofCrystals>=10
				DisplayText(" de "+Str$(NofTotalCrystals),9,0,1,5)
			Else 
				DisplayText(" de "+Str$(NofTotalCrystals),8,0,1,5)
			EndIf

		EndIf
	Else
		DisplayText(" ",0,0,1,0)
		DisplayText(" ",0,0,1,5)

	EndIf
	
	
	
	; Score
	;ex$="Level Score: "+Str$(CurrentLevelScore+MapTime)
	;DisplayText(ex$,50-Len(ex$),0,1,4)
	
	; Time
	If LevelCompleted=0 And NoTimeLimit=0
		i=MilliSecs()
		MapTimeElapsed=i-MapMilliTime
		
		If MapTimeElapsed>=1000
			MapTime=MapTime-1
			If MapTime=30 Then PlayMySound(SFXTime,0)
			For j=0 To 10
				If MapTime=j Then PlayMySound(SFXCount(j),0)
			Next
			MapMilliTime=i
		EndIf
		If MapTime<0 
			KillStinky()
			If LoofExists=True Then KillLoof()
			MapTime=0
		EndIf
	EndIf
	MapTimeMinutes=MapTime / 60
	MapTimeSeconds=MapTime Mod 60
	MapTimeString$=Str$(MapTimeMinutes)+":"
	If MapTimeSeconds<10
		MapTimeString$=MapTimeString$+"0"+Str$(MapTimeSeconds)
	Else
		MapTimeString$=MapTimeString$+Str$(MapTimeSeconds)
	EndIf
	DisplayText(MapTimeString$,50-Len(MapTimeString$),0,1,1)

	; Fps (optional)
	If ShowFPS=True
		DisplayText("FPS: "+Str$(fps),0,1,1,3)
	Else
		DisplayText("",0,1,1,3)
	EndIf
	
	
	
	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next
	
	
	
	
	
			
;	Text 500,20, "FPS: "+Str$(StinkyMovingX)
;	Text 500,40, "FPS: "+Str$(SFXElectroVolume)

	
	;If EntityY(Camera,True)>50
	;	EndLevel()
	;	StartMenu(0)
	;EndIf
	
	Flip
	
	If KeyHit(66) Then
		SaveBuffer(FrontBuffer(),"screenshot.bmp")
	End If



	
	If LevelCompleted=2 ; totally over, but restart menu
		EndLevel()
		CurrentLevel=CurrentLevel+1
		If CurrentWorld=PlayGroundLevel/10
			StartMenu(8)
		Else
			StartMenu(4)
		EndIf
	Else If LevelCompleted=3 	; restart same level
		EndLevel()
		StartLevel(CurrentLevel)
	EndIf


	
End Function




Function UpdateGame()

	For i=0 To 10
		SFXSingleVolume(i)=0 ; used to figure out the loudest of the "single" sounds
	Next
	
		
	; INPUT CONTROLS
	; ---------------
;	If KeyDown(2)
		;test idle animations
;		AnimateMD2 StinkyEntity,1,0.2,44,52
;		StinkyAnimCurrent=4
;	EndIf
;	If KeyDown(3)
;		;test idle animations
;		AnimateMD2 StinkyEntity,1,0.2,53,61
;		StinkyAnimCurrent=5
;	EndIf
;	If KeyDown(4)
;		;test idle animations
;		AnimateMD2 StinkyEntity,1,0.2,62,104
;		StinkyAnimCurrent=6
;	EndIf
;	If KeyDown(5)
;		;test idle animations
;		AnimateMD2 StinkyEntity,1,0.2,105,147
;		StinkyAnimCurrent=7
;
;		;EntityTexture StinkyEntity,StinkyTextureB
;	EndIf
;	If KeyDown(6)
;		EntityTexture StinkyEntity,StinkyTextureC
;	EndIf
;	If KeyDown(7)
;		EntityTexture StinkyEntity,StinkyTextureD
;	EndIf
;	If KeyDown(8)
;		AnimateMD2 StinkyEntity,0,1,0,0
;		EntityTexture StinkyEntity,StinkyTexture
;		StinkyAnimNew=0
;		StinkyAnimCurrent=0
;	EndIf
	
	
	If KeyDown(56) And KeyDown(45) 
		; Alt-X + cursor keys - Camera Cheat Control
		If KeyDown(72) 
			;If EntityZ(Camera)<-1-9.5
				TranslateEntity Camera,0,0,.1	;forward
			;	FreeCameraZ=FreeCameraZ+.1
			;EndIf
		EndIf
		If KeyDown(80) 
			;If EntityZ(Camera)>-(MapHeight-1)-9.5
				TranslateEntity Camera,0,0,-.1	;backward
			;	FreeCameraZ=FreeCameraZ-.1
			;EndIf
		EndIf
		If KeyDown(75)
			;If EntityX(Camera)>1
				TranslateEntity Camera,-.1,0,0	;left
			;	FreeCameraX=FreeCameraX-.1
			;EndIf
		EndIf
		If KeyDown(77)
			;If EntityX(Camera)<MapWidth-1
				TranslateEntity Camera,.1,0,0	;right
			;	FreeCameraX=FreeCameraX+.1
			;EndIf
		EndIf
		If KeyDown(73)
			;If EntityX(Camera)>1
				TranslateEntity Camera,0,.1,0	;up
			;	FreeCameraX=FreeCameraX-.1
			;EndIf
		EndIf
		If KeyDown(81)
			;If EntityX(Camera)<MapWidth-1
				TranslateEntity Camera,0,-.1,0	;down
			;	FreeCameraX=FreeCameraX+.1
			;EndIf
		EndIf
		If KeyDown(71)
			TurnEntity Camera,1,0,0
		EndIf
		If KeyDown(79)
			TurnEntity Camera,-1,0,0
		EndIf
	
	
	Else If KeyDown(42) Or KeyDown(54)
		; shift key is pressed - Camera control
		If ShiftKeyPressed=False
			For i=11 To 15
				StopSign(i)
			Next
		EndIf
		
		ShiftKeyPressed=True
		
	
		If KeyDown(200) Or KeyDown(72) 
			If EntityZ(Camera)<-1-9.5
				TranslateEntity Camera,0,0,.1	;forward
				FreeCameraZ=FreeCameraZ+.1
			EndIf
		EndIf
		If KeyDown(208) Or KeyDown(80) 
			If EntityZ(Camera)>-(MapHeight-1)-9.5
				TranslateEntity Camera,0,0,-.1	;backward
				FreeCameraZ=FreeCameraZ-.1
			EndIf
		EndIf
		If KeyDown(75) Or KeyDown(203)
			If EntityX(Camera)>1
				TranslateEntity Camera,-.1,0,0	;left
				FreeCameraX=FreeCameraX-.1
			EndIf
		EndIf
		If KeyDown(77) Or KeyDown(205)
			If EntityX(Camera)<MapWidth-1
				TranslateEntity Camera,.1,0,0	;right
				FreeCameraX=FreeCameraX+.1
			EndIf
		EndIf
		
			;If KeyDown(73) TranslateEntity Camera,0,.1,0	;up
		;If KeyDown(81) TranslateEntity Camera,0,-.1,0	;down
			
	Else
		If ShiftKeyPressed=True
			CheckSigns()
		EndIf
		
		ShiftKeyPressed=False
		If FreeCameraX<>0 Or FreeCameraZ<>0
			TranslateEntity Camera,-FreeCameraX,0,-FreeCameraZ
			FreeCameraX=0
			FreeCameraZ=0
		EndIf
		; no shift - stinky control
		; first check if we accept any input?
		If StinkyDying=False And StinkyDead=False And LoofDying=False And LoofDead=False And LevelCompleted=0 And FocusSwitchTimer<20
		 ; up
		 If KeyDown(200) Or KeyDown(72)
			If FocusOnStinky=True
				If StinkyMoving=False And (MapTile(StinkyX,StinkyY)/100<>5 Or ConveyorActive=False)
					If CanStinkyMove(0,-1)=True	Then MoveStinky(0,-1)
				EndIf
			Else
				If LoofMoving=False And (MapTile(LoofX,LoofY)/100<>5 Or ConveyorActive=False)
					If CanLoofMove(0,-1)=True	 Then MoveLoof(0,-1)
				EndIf
			EndIf
		 EndIf
		 ; down
		 If KeyDown(208) Or KeyDown(80)
			If FocusOnStinky=True
				If StinkyMoving=False And (MapTile(StinkyX,StinkyY)/100<>5 Or ConveyorActive=False)
					If CanStinkyMove(0,1)=True Then MoveStinky(0,1)
				EndIf
			Else
				If LoofMoving=False And (MapTile(LoofX,LoofY)/100<>5 Or ConveyorActive=False)
					If CanLoofMove(0,1)=True Then MoveLoof(0,1)
				EndIf
			EndIf
		 EndIf
		 ; left
		 If KeyDown(203) Or KeyDown(75)
			If FocusOnStinky=True
				If StinkyMoving=False And (MapTile(StinkyX,StinkyY)/100<>5 Or ConveyorActive=False)
					If CanStinkyMove(-1,0)=True Then MoveStinky(-1,0)
				EndIf
			Else
				If LoofMoving=False And (MapTile(LoofX,LoofY)/100<>5 Or ConveyorActive=False)
					If CanLoofMove(-1,0)=True Then MoveLoof(-1,0)
				EndIf
			EndIf
		 EndIf
		 ; right
		 If KeyDown(205) Or KeyDown(77)
			If FocusOnStinky=True
				If StinkyMoving=False And (MapTile(StinkyX,StinkyY)/100<>5 Or ConveyorActive=False)
					If CanStinkyMove(1,0)=True Then MoveStinky(1,0)
				EndIf
			Else
				If LoofMoving=False And (MapTile(LoofX,LoofY)/100<>5 Or ConveyorActive=False)
					If CanLoofMove(1,0)=True Then MoveLoof(1,0)
				EndIf
			EndIf
		 EndIf
	EndIf
	
	 ; Return - Switch Stinky & Loof
	 If (KeyDown(28) Or KeyDown(156)) And FocusSwitchTimer=0
		SwitchStinkyLoof()
	 EndIf
	EndIf
	If FocusSwitchTimer>0
		FocusSwitchTimer=FocusSwitchTimer-1
	EndIf
	
	; Stinky Walking Animation End?
	; needs to be between Move and Endmove (i.e.here) to prevent animation from stopping if a new tile
	; is reached (and hence endmove is called)
	If StinkyMoving=False And StinkyAnimCurrent<3
		;AnimateMD2 StinkyEntity,1,0.1,105,147

		AnimateMD2 StinkyEntity,0,1,0,0
		StinkyAnimNew=0
		StinkyAnimCurrent=0
	EndIf
	If LoofExists=True And LoofMoving=False And LoofAnimCurrent<3
		AnimateMD2 LoofEntity,0,1,0,0
		LoofAnimNew=0
		LoofAnimCurrent=0
	EndIf
	
	If (StinkyTexture=StinkyBeforeBlinkTexture)
		StinkyBlinkCounter=StinkyBlinkCounter-1
		If StinkyBlinkCounter>20 And StinkyBlinkCounter<30
			EntityTexture StinkyEntity,StinkyTextureD
			StinkyIdleCounter=StinkyIdleCounter-1
		EndIf
		If StinkyBlinkCounter=>10 And StinkyBlinkCounter<20
			EntityTexture StinkyEntity,StinkyTexture
			StinkyIdleCounter=StinkyIdleCounter-1
		EndIf
		If StinkyBlinkCounter>0 And StinkyBlinkCounter<10
			EntityTexture StinkyEntity,StinkyTextureD
			StinkyIdleCounter=StinkyIdleCounter-1
		EndIf
		If StinkyBlinkCounter<=0
			EntityTexture StinkyEntity,StinkyTexture
			StinkyBlinkCounter=500+Rand(0,500)
		EndIf
	EndIf
	If (LoofTexture=LoofBeforeBlinkTexture) And LoofExists=True
		LoofBlinkCounter=LoofBlinkCounter-1
		If LoofBlinkCounter>20 And LoofBlinkCounter<30
			EntityTexture LoofEntity,LoofTextureD
			LoofIdleCounter=LoofIdleCounter-1
		EndIf
		If LoofBlinkCounter=>10 And LoofBlinkCounter<20
			EntityTexture LoofEntity,LoofTexture
			LoofIdleCounter=LoofIdleCounter-1
		EndIf
		If LoofBlinkCounter>0 And LoofBlinkCounter<10
			EntityTexture LoofEntity,LoofTextureD
			LoofIdleCounter=LoofIdleCounter-1
		EndIf
		If LoofBlinkCounter<=0
			EntityTexture LoofEntity,LoofTexture
			LoofBlinkCounter=500+Rand(0,500)
		EndIf
	EndIf

		
	
	StinkyIdleCounter=StinkyIdleCounter+1
	LoofIdleCounter=LoofIdleCounter+1
	
	

	If StinkyIdleCounter=900
		i=Rand(0,100)
		j=Rand(0,100)
		If i<50
			AnimateMD2 StinkyEntity,0,1,0,0
			StinkyAnimNew=0
			StinkyAnimCurrent=0
			StinkyIdleCounter=600
			If j<60
				EntityTexture StinkyEntity,StinkyTexture
				StinkyBeforeBlinkTexture=StinkyTexture
			Else If j<95
				EntityTexture StinkyEntity,StinkyTextureC
				StinkyBeforeBlinkTexture=StinkyTextureC
			Else 
				EntityTexture StinkyEntity,StinkyTextureD
				StinkyBeforeBlinkTexture=StinkyTextureD
			EndIf

		Else If i<60
			AnimateMD2 StinkyEntity,1,0.1,105,147
			StinkyAnimNew=7
			StinkyAnimCurrent=7
			StinkyIdleCounter=-300
			If j<50
				EntityTexture StinkyEntity,StinkyTexture
				StinkyBeforeBlinkTexture=StinkyTexture
			Else If j<95
				EntityTexture StinkyEntity,StinkyTextureC
				StinkyBeforeBlinkTexture=StinkyTextureC
			Else 
				EntityTexture StinkyEntity,StinkyTextureD
				StinkyBeforeBlinkTexture=StinkyTextureD
			EndIf

		Else If i<65
			AnimateMD2 StinkyEntity,1,0.1,62,104
			StinkyAnimCurrent=6
			StinkyAnimNew=6
			StinkyIdleCounter=-300
			If j<50
				EntityTexture StinkyEntity,StinkyTexture
				StinkyBeforeBlinkTexture=StinkyTexture
			Else 
				EntityTexture StinkyEntity,StinkyTextureC
				StinkyBeforeBlinkTexture=StinkyTextureC
			EndIf

		Else 
			AnimateMD2 StinkyEntity,1,0.2,53,61
			StinkyAnimCurrent=5
			StinkyAnimCurrent=5
			StinkyIdleCounter=-300
			If j<45
				EntityTexture StinkyEntity,StinkyTexture
				StinkyBeforeBlinkTexture=StinkyTexture
			Else If j<95
				EntityTexture StinkyEntity,StinkyTextureC
				StinkyBeforeBlinkTexture=StinkyTextureC
			Else 
				EntityTexture StinkyEntity,StinkyTextureD
				StinkyBeforeBlinkTexture=StinkyTextureD
			EndIf

		EndIf
		
	EndIf
	If LoofIdleCounter=1200 And LoofExists=True
		i=Rand(0,100)
		j=Rand(0,100)
		If i<50
			AnimateMD2 LoofEntity,0,1,0,0
			LoofAnimNew=0
			LoofAnimCurrent=0
			LoofIdleCounter=800
			If j<60
				EntityTexture LoofEntity,LoofTexture
				LoofBeforeBlinkTexture=LoofTexture
			Else If j<95
				EntityTexture LoofEntity,LoofTextureC
				LoofBeforeBlinkTexture=LoofTextureC
			Else 
				EntityTexture LoofEntity,LoofTextureD
				LoofBeforeBlinkTexture=LoofTextureD
			EndIf

		Else If i<60
			AnimateMD2 LoofEntity,1,0.2,105,147
			LoofAnimNew=7
			LoofAnimCurrent=7
			LoofIdleCounter=-100
			If j<50
				EntityTexture LoofEntity,LoofTexture
				LoofBeforeBlinkTexture=LoofTexture
			Else If j<90
				EntityTexture LoofEntity,LoofTextureC
				LoofBeforeBlinkTexture=LoofTextureC
			Else 
				EntityTexture LoofEntity,LoofTextureD
				LoofBeforeBlinkTexture=LoofTextureD
			EndIf

		Else If i<65
			AnimateMD2 LoofEntity,1,0.2,62,104
			LoofAnimCurrent=6
			LoofAnimNew=6
			LoofIdleCounter=-100
			If j<50
				EntityTexture LoofEntity,LoofTexture
				LoofBeforeBlinkTexture=LoofTexture
			Else 
				EntityTexture LoofEntity,LoofTextureC
				LoofBeforeBlinkTexture=LoofTextureC
			EndIf

		Else 
			AnimateMD2 LoofEntity,1,0.25,53,61
			LoofAnimCurrent=5
			LoofAnimCurrent=5
			LoofIdleCounter=-400
			If j<30
				EntityTexture LoofEntity,LoofTexture
				LoofBeforeBlinkTexture=LoofTexture
			Else If j<80
				EntityTexture LoofEntity,LoofTextureC
				LoofBeforeBlinkTexture=LoofTextureC
			Else 
				EntityTexture LoofEntity,LoofTextureD
				LoofBeforeBlinkTexture=LoofTextureD
			EndIf

		EndIf
		
	EndIf

		
	

			


	
	; Stinky dying animation
	If StinkyDying=True And StinkyDead=False
		TranslateEntity StinkyEntity,0,Float(StinkyDyingTimer)/2000.0,0
		
		oldx#=EntityX(StinkyEntity)
		oldy#=EntityY(StinkyEntity)
		oldz#=EntityZ(StinkyEntity)
		;PositionEntity StinkyEntity,0,-0.6,0
		TurnEntity StinkyEntity,4,6,0
		;PositionEntity StinkyEntity,oldx,oldy,oldz
		
		
		TranslateEntity Camera,0,Float(StinkyDyingTimer)/2000.0,0

		StinkyDyingTimer=StinkyDyingTimer-1
		If StinkyDyingTimer=0
			StinkyDyingTimer=320
			;CameraFogColor camera,0,0,0
			StinkyDead=True
			If LoofDead=False PlayMySound(SFXLevelLose,0)
			HideEntity StinkyEntity
			; and explode
			For i=0 To 30*DetailLevel	
				; angle of particle
				angle#=Rnd(0,359)
				speed#=Rnd(.03,.05)
				dx#=speed*Sin(angle) 
				dz#=speed*Cos(angle)
				CreateParticle(0,Rand(5,8),.2,EntityX(StinkyEntity),EntityY(StinkyEntity),EntityZ(StinkyEntity),dx,Rnd(-.001,.001),dz,3,0,200)
			Next
		EndIf
	EndIf
	If StinkyDead=True
		; fade out
		StinkyDyingTimer=StinkyDyingTimer-1
		If StinkyDyingTimer<160
			LightColor Levellight,-255+510*StinkyDyingTimer/160,-255+510*StinkyDyingTimer/160,-255+510*StinkyDyingTimer/160
			AmbientLight 255*StinkyDyingTimer/160,255*StinkyDyingTimer/160,255*StinkyDyingTimer/160
		EndIf
		If StinkyDyingTimer<80
		;	CameraFogRange Camera,StinkyDyingTimer,StinkyDyingTimer*6
		EndIf
;		CameraViewport camera,320-StinkyDyingTimer,240-StinkyDyingTimer*.75,StinkyDyingTimer*2,StinkyDyingTimer*1.5
		
		If StinkyDyingTimer=0
			; and done - restart level
			
			LevelCompleted=3
		EndIf
	EndIf
	
	; Loof dying animation
	If LoofDying=True And LoofDead=False
		TranslateEntity LoofEntity,0,Float(LoofDyingTimer)/2000.0,0
		
		oldx#=EntityX(LoofEntity)
		oldy#=EntityY(LoofEntity)
		oldz#=EntityZ(LoofEntity)
		;PositionEntity LoofEntity,0,-0.6,0
		TurnEntity LoofEntity,4,6,0
		;PositionEntity LoofEntity,oldx,oldy,oldz
		
		
		TranslateEntity Camera,0,Float(LoofDyingTimer)/2000.0,0

		LoofDyingTimer=LoofDyingTimer-1
		If LoofDyingTimer=0
			LoofDyingTimer=320
			;CameraFogColor camera,0,0,0
			LoofDead=True
			If StinkyDead=False PlayMySound(SFXLevelLose,0)
			HideEntity LoofEntity
			; and explode
			For i=0 To 30*DetailLevel	
				; angle of particle
				angle#=Rnd(0,359)
				speed#=Rnd(.03,.05)
				dx#=speed*Sin(angle) 
				dz#=speed*Cos(angle)
				CreateParticle(0,Rand(5,8),.2,EntityX(LoofEntity),EntityY(LoofEntity),EntityZ(LoofEntity),dx,Rnd(-.001,.001),dz,3,0,200)
			Next
		EndIf
	EndIf
	If LoofDead=True
		; fade out
		LoofDyingTimer=LoofDyingTimer-1
		If LoofDyingTimer<160
			LightColor Levellight,-255+510*LoofDyingTimer/160,-255+510*LoofDyingTimer/160,-255+510*LoofDyingTimer/160
			AmbientLight 255*LoofDyingTimer/160,255*LoofDyingTimer/160,255*LoofDyingTimer/160
		EndIf
		If LoofDyingTimer<80
		;	CameraFogRange Camera,LoofDyingTimer,LoofDyingTimer*6
		EndIf
;		CameraViewport camera,320-LoofDyingTimer,240-LoofDyingTimer*.75,LoofDyingTimer*2,LoofDyingTimer*1.5
		
		If LoofDyingTimer=0
			; and done - restart level
			
			LevelCompleted=3
		EndIf
	EndIf

	
	
	

	
	
	
	
	
	
	If LevelTimer<160
		; fade in
		If SignActive=0
			LightColor Levellight,-255+510*LevelTimer/160,-255+510*LevelTimer/160,-255+510*LevelTimer/160
			AmbientLight 255*LevelTimer/160,255*LevelTimer/160,255*LevelTimer/160
		EndIf
		;AmbientLight 255,255,255
			
		If LevelTimer<100
		;	CameraFogColor camera,0,192*LevelTimer/100,255*LevelTimer/100
		;	CameraClsColor camera,0,192*LevelTimer/100,255*LevelTimer/100
		;	CameraFogRange Camera,LevelTimer,LevelTimer*2
		EndIf
		
	;	If LevelTimer>100
			TranslateEntity Camera,0,-10.0/160.0,-10.0/160.0
			TurnEntity Camera,-35.0/160.0,0,0
			
			; ScaleEntity StinkyWarpSphere,Float(160-LevelTimer)/60.0,Float(160-LevelTimer)/60.0,Float(160-LevelTimer)/60.0
		;	EntityAlpha StinkyWarpSphere,Float(160-LevelTimer)/60.0
		;	If LoofExists=True Then EntityAlpha LoofWarpSphere,Float(160-LevelTimer)/60.0

	;	EndIf		
	EndIf
	;If LevelTimer=160
	;	HideEntity StinkyWarpSphere
	;	If LoofExists=True Then	HideEntity LoofWarpSphere
	;EndIf
	

			
		
		
	
	;standing on conveyor... check if path has been cleared and movement can be made
	If StinkyMoving=False And MapTile(StinkyX,StinkyY)/100=5 And ConveyorActive=True
		If MapTile(StinkyX,StinkyY)=500 
			If CanStinkyMove(0,-1)=True Then MoveStinky(0,-1)
		EndIf
		If MapTile(StinkyX,StinkyY)=502 
			If CanStinkyMove(0,1)=True Then MoveStinky(0,1)
		EndIf
		If MapTile(StinkyX,StinkyY)=501 
			If CanStinkyMove(1,0)=True Then MoveStinky(1,0)
		EndIf
		If MapTile(StinkyX,StinkyY)=503 
			If CanStinkyMove(-1,0)=True Then MoveStinky(-1,0)
		EndIf
	EndIf
	If LoofExists=True And LoofMoving=False And MapTile(LoofX,LoofY)/100=5 And ConveyorActive=True
		If MapTile(LoofX,LoofY)=500 
			If CanLoofMove(0,-1)=True Then MoveLoof(0,-1)
		EndIf
		If MapTile(LoofX,LoofY)=502 
			If CanLoofMove(0,1)=True Then MoveLoof(0,1)
		EndIf
		If MapTile(LoofX,LoofY)=501 
			If CanLoofMove(1,0)=True Then MoveLoof(1,0)
		EndIf
		If MapTile(LoofX,LoofY)=503 
			If CanLoofMove(-1,0)=True Then MoveLoof(-1,0)
		EndIf
	EndIf


	; --------------
	;   BOXES
	; --------------
	For i=0 To NofBoxes-1
		;standing on conveyor... check if path has been cleared and movement can be made
		If BoxMoving(i)=False And MapTile(BoxX(i),BoxY(i))/100=5 And ConveyorActive=True
			If MapTile(BoxX(i),BoxY(i))=500 
				If CanBoxMove(i,0,-1)=True Then MoveBox(i,0,-1,0)
			EndIf
			If MapTile(BoxX(i),BoxY(i))=502 
				If CanBoxMove(i,0,1)=True Then MoveBox(i,0,1,0)
			EndIf
			If MapTile(BoxX(i),BoxY(i))=501 
				If CanBoxMove(i,1,0)=True Then MoveBox(i,1,0,0)
			EndIf
			If MapTile(BoxX(i),BoxY(i))=503 
				If CanBoxMove(i,-1,0)=True Then MoveBox(i,-1,0,0)
			EndIf
		EndIf
	Next
	
	; -------------------
	; 	MONSTER CONTROL
	; -------------------
	ControlFireballs()
	ControlStinky()
	ControlLoof()
	; Box Swaying
	ControlBoxes()
	; ElectroTiles
	ControlElectroTiles()
	; Cannons & Fireballs
	ControlCannons()
	
	; Timer Buttons
	ControlTimerButtons()
	; Monsters
	ControlMonsters()
	; Items
	ControlItems()
	; Fish
	ControlFish()
	; Particles
	ControlParticles()
	ControlFireballs()
	
	
	; OBJECT MOVEMENTS
	; ----------------
	If StinkyMoving=True And StinkyDead=False And StinkyDying=False
		TranslateEntity StinkyEntity,StinkyMovingX/Float(StinkySpeed),0,-StinkyMovingY/Float(StinkySpeed)
		TranslateEntity StinkyShadowEntity,StinkyMovingX/Float(StinkySpeed),0,-StinkyMovingY/Float(StinkySpeed)
		If FocusOnStinky=True
			TranslateEntity Camera,StinkyMovingX/Float(StinkySpeed),0,-StinkyMovingY/Float(StinkySpeed)
		EndIf
		StinkyMovingTimer=StinkyMovingTimer-1
	EndIf
	If LoofMoving=True And LoofDead=False And LoofDying=False
		TranslateEntity LoofEntity,LoofMovingX/Float(LoofSpeed),0,-LoofMovingY/Float(LoofSpeed)
		TranslateEntity LoofShadowEntity,LoofMovingX/Float(LoofSpeed),0,-LoofMovingY/Float(LoofSpeed)
		If FocusOnStinky=False
			TranslateEntity Camera,LoofMovingX/Float(LoofSpeed),0,-LoofMovingY/Float(LoofSpeed)
		EndIf
		LoofMovingTimer=LoofMovingTimer-1
	EndIf

	For i=0 To NofBoxes-1
		If BoxMoving(i)=True
			TranslateEntity BoxEntity(i),BoxMovingX(i)/Float(BoxSpeed(i)),0,-BoxMovingY(i)/Float(BoxSpeed(i))
			BoxMovingTimer(i)=BoxMovingTimer(i)-1
			
		EndIf
	Next
	For i=0 To NofMonsters-1
		If MonsterMoving(i)=True
			TranslateEntity MonsterEntity(i),MonsterMovingX(i)/Float(MonsterSpeed(i)),0,-MonsterMovingY(i)/Float(MonsterSpeed(i))
			TranslateEntity MonsterShadowEntity(i),MonsterMovingX(i)/Float(MonsterSpeed(i)),0,-MonsterMovingY(i)/Float(MonsterSpeed(i))
			MonsterMovingTimer(i)=MonsterMovingTimer(i)-1
			
		EndIf
		
	Next
	
	; MOVEMENT END?
	; ------------
	If StinkyMoving=True And StinkyMovingTimer=0
		EndMoveStinky()
	EndIf
	If LoofMoving=True And LoofMovingTimer=0
		EndMoveLoof()
	EndIf

	For i=0 To NofBoxes-1
		If BoxMoving(i)=True And BoxMovingTimer(i)=0
			EndMoveBox(i)
		EndIf
	Next
	For i=0 To NofMonsters-1
		If MonsterMoving(i)=True And MonsterMovingTimer(i)=0
			EndMoveMonster(i)
		EndIf
	Next



	
	
	
	; OBJECT CLEANUP
	; --------------
	CleanUpBoxes()
	CleanUpFireballs()
	CleanUpMonsters()
	CleanUpItems()
	
	
		
	


		
		
	; ---------------
	; SPECIAL EFFECTS
	; ---------------
	
	; Camera Scroll?
	; e.g. after switch between stinky and loof
	If CameraDX>=CameraDStep
		TranslateEntity Camera,CameraDStep,0,0
		CameraDX=CameraDX-CameraDStep
	Else If CameraDX<=-CameraDStep
		TranslateEntity Camera,-CameraDStep,0,0
		CameraDX=CameraDX+CameraDStep
	Else
		TranslateEntity Camera,CameraDX,0,0
		CameraDX=0
	EndIf
	If CameraDZ>=CameraDStep
		TranslateEntity Camera,0,0,CameraDStep
		CameraDZ=CameraDZ-CameraDStep
	Else If CameraDZ<=-CameraDStep
		TranslateEntity Camera,0,0,-CameraDStep
		CameraDZ=CameraDZ+CameraDStep
	Else
		TranslateEntity Camera,0,0,CameraDZ
		CameraDZ=0
	EndIf

	


		
	
	; scroll water texture
	Watertexu#=Watertexu#+0.001
	Watertexv#=Watertexv#+0.0005
	;Watertexr#=Watertexr#+1
	If Watertexu#=1.0 Then Watertexu#=0.0
	If Watertexv#=1.0 Then Watertexv#=0.0	
	;If Watertexr#=360.0 Then Watertexr#=0.0
	PositionTexture WaterTexture,Watertexu#,Watertexv#
	If Version>=10 	PositionTexture LavaTexture,Watertexu#,Watertexv#
	PositionTexture RainbowTexture,Watertexu,WaterTexv*10
	
	If MapBackGround=0 ; scroll sky
		PositionEntity BackGroundPlane,WaterTexu*40,-50,WaterTexv*40
		PositionEntity BackGroundPlane2,WaterTexu*20,-60,WaterTexv*20

	EndIf

	;RotateTexture WaterTexture,90
	;PositionTexture LaserGateTexture(0),Watertexu#,Watertexv#

	;Lasergates
	;PositionTexture LaserGateTexture(1),0,Watertexv*10;2*Sin(Watertexv*720*2)
	For i=0 To NofLaserGates-1
		If LaserGateActive(LaserGateCol(i))=True And LaserGateCol(i)>0
			TurnEntity LaserGateEntity(i),1,0,0
			TurnEntity LaserGateEntity2(i),1,0,0

		EndIf
	Next
	
	

	; scroll conveyor textures
	If ConveyorActive=True
		conveyortex#=conveyortex#+(1.0/BaseSpeed)*2
		If conveyortex#>=1.0 Then conveyortex#=conveyortex#-1
	EndIf
	PositionTexture ConveyorTexture1,0,1.0-conveyortex#
	PositionTexture ConveyorTexture2,0,1.0-conveyortex#
	PositionTexture ConveyorTexture3,0,1.0-conveyortex#
	PositionTexture ConveyorTexture4,0,1.0-conveyortex#
	
	

	
	; Level Completed Zoom
	If LevelCompleted=1
		LevelCompletedTimer=LevelCompletedTimer+1
	
		TranslateEntity Camera,0,10.0/512.0,10.0/512.0
		TurnEntity Camera,45.0/512.0,0,0
		If LevelCompletedTimer=510
			LevelCompleted=2
		EndIf
		
		AmbientLight 255-LevelCompletedTimer/2,255-LevelCompletedTimer/2,255-LevelCompletedTimer/2
		LightColor Levellight,255-LevelCompletedTimer*1,255-1*LevelCompletedTimer,255-1*LevelCompletedTimer

		
	EndIf
	
	; Final Gate Anim
	If FinalGateAnim>0
		FinalGateAnim=FinalGateAnim-1
		For k=0 To NofLaserGates-1
			If LaserGateCol(k)=0 
				TranslateEntity LaserGateEntity(k),0,-0.016,0
				If FinalGateAnim=0
					HideEntity LaserGateEntity(k)
				EndIf
			EndIf
		Next
	EndIf
	
	; Stinky/Loof Shrink in Rainbowgate
	If StinkyShrink>0
		shrinkfactor#=Float(StinkyShrink)/60.0
		StinkyShrink=StinkyShrink-1
		ScaleEntity StinkyEntity,shrinkfactor,shrinkfactor,shrinkfactor
		If StinkyShrink=0 Then HideEntity StinkyEntity
	EndIf
	If LoofShrink>0
		shrinkfactor#=Float(LoofShrink)/60.0
		LoofShrink=LoofShrink-1
		ScaleEntity LoofEntity,shrinkfactor,shrinkfactor,shrinkfactor
		If LoofShrink=0 Then HideEntity LoofEntity
	EndIf


	
		
		
	; single sounds (electro, cannon)
	For i=0 To 10
		If SFXSingleVolume(i)>0
			SoundVolume SFXSingle(i),OptionsSoundLevel*SFXSingleVolume(i)
			PlaySound SFXSingle(i)
		EndIf
	Next
	
	; Signs
	ControlSigns()
			
	If Snow=True
		If LevelTimer Mod (1+(5-DetailLevel)*(5-DetailLevel))=0
			CreateParticle(0,10,.05,Rnd(-2,MapWidth+2),Rnd(5,10),-Rnd(-2,MapHeight+2),.01,-Rnd(.02,.03),-.01,Rnd(-.5,.25),0,10000)
			
		EndIf
	EndIf
	


	

	
		
	LevelTimer=LevelTimer+1
	
	
	
	If Version<>11
		;Cheats (CTRL + F-Keys)
		If (KeyDown(29) Or KeyDown(157)) 
			If KeyDown(63) 	ShowFPS=True ;F5
			If KeyDown(64) 	ShowFPS=False ;F6
			If KeyDown(67)	;F9
				LevelCompleted=1
				LevelCompletedTimer=0
				If Language=0
					ex$="WELL DONE!!! LEVEL COMPLETED!!!"
					ex$=ex$+"#... NOT! :) # "
				Else If language=2
					ex$="BRAVO!!!#NIVEAU COMPLETE!!!#...PAS VRAIMENT! :)# #TRICHE...TRICHE...TRICHE"
				Else 
					ex$="CHEAT...CHEAT...CHEAT"
				EndIf
				CurrentLevelScore=1
				If Language<>2
				ex$=ex$+"#TOTAL SCORE: "+Str$(CurrentLevelScore)
				
				If CurrentPlayerLevelScore(CurrentLevel)>0 
					If CurrentLevelScore>CurrentPlayerLevelScore(CurrentLevel)
						ex$=ex$+"#!!! NEW LEVEL RECORD !!!"
					Else
						ex$=ex$+"#NO NEW RECORD."
					EndIf
				EndIf
				EndIf
				If CurrentLevelScore>CurrentPlayerLevelScore(CurrentLevel) 
					CurrentPlayerLevelScore(CurrentLevel)=CurrentLevelScore
				EndIf
				
				StartSign(ex$,7,2,1)
				
				If CurrentLevel<PlayGroundLevel Then SavePlayer()
			EndIf
		EndIf
	Else
		;Cheats (1 + 2/3/4) for Review Copy
		If (KeyDown(2)) 
			If KeyDown(3) 	ShowFPS=True
			If KeyDown(4) 	ShowFPS=False
			If KeyDown(5)	
				LevelCompleted=1
				LevelCompletedTimer=0
				a=Rand(0,100)
				If a<20
					ex$="WELL DONE!!! LEVEL COMPLETED!!!"
				Else If a<40
					ex$="CONGRATULATIONS!!! LEVEL COMPLETED!!!"
				Else If a<60
					ex$="EXCELLENT!!! LEVEL COMPLETED!!!"
				Else If a<80
					ex$="GOOD JOB!!! LEVEL COMPLETED!!!"
				Else 
					ex$="NICELY DONE!!! LEVEL COMPLETED!!!"
				EndIf
				ex$=ex$+"#... NOT! :) # "
				CurrentLevelScore=1
				ex$=ex$+"#TOTAL SCORE: "+Str$(CurrentLevelScore)
				
				If CurrentPlayerLevelScore(CurrentLevel)>0 
					If CurrentLevelScore>CurrentPlayerLevelScore(CurrentLevel)
						ex$=ex$+"#!!! NEW LEVEL RECORD !!!"
					Else
						ex$=ex$+"#NO NEW RECORD."
					EndIf
				EndIf
				If CurrentLevelScore>CurrentPlayerLevelScore(CurrentLevel) 
					CurrentPlayerLevelScore(CurrentLevel)=CurrentLevelScore
				EndIf
				
				StartSign(ex$,7,2,1)
				
				If CurrentLevel<PlayGroundLevel Then SavePlayer()
			EndIf
		EndIf

	
	
	
	EndIf
	
	

End Function



; Function Collections
;Include "d:\blitz3d\terapak\bones2.bb"
Include "level.bb"
Include "setup.bb"
Include "gameobject.bb"
Include "text.bb"
Include "particles.bb"
Include "sound.bb"
Include "menu.bb"
Include "error.bb"
Include "pack.bb"