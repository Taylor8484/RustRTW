; Replay recorder for the Rust port (docs/architecture.md, "Recordings").
;
; Off unless the game is started with -record on its command line (in the
; Blitz3D IDE: Program > Program Command Line). It then writes
; "replay <date> <time>.wlr" next to the game: a text file with the random
; seed and options at each level start, and per tick the keys held and the
; state of each object family, so the port can replay the input and report
; the first tick where each family differs.
;
; Nothing here draws random numbers or changes game state.

Global RecOn=Instr(Lower$(CommandLine$()),"-record")>0
Global RecFile=0
Global RecBank=CreateBank(4)
Global RecSeedBefore, RecInput
Global RecFrameMs=-1	; MilliSecs() at this frame's time-limit check, -1 if none

Function RecOpen()
	Local name$
	name$="replay "+CurrentDate$()+" "+Replace$(CurrentTime$(),":","-")+".wlr"
	RecFile=WriteFile(name$)
	If RecFile=0
		RecOn=False
		Return
	EndIf
	WriteLine RecFile,"WLREC 1 RTWP"
End Function

Function RecClose()
	If RecFile<>0 Then CloseFile RecFile
	RecFile=0
End Function

; A float's bits as an int, so the log holds it exactly.
Function RecBits(f#)
	PokeFloat RecBank,0,f
	Return PeekInt(RecBank,0)
End Function

; At the top of StartLevel: what the port needs to start the same level.
; The path is the file LoadLevel opens.
Function RecStartLevel(Level)
	Local path$
	If RecOn=False Then Return
	If RecFile=0 Then RecOpen()
	If RecOn=False Then Return
	If Level=-999
		path$="CustomLevels\"+CurrentExtraLevelDirectory$+FileDir$(CurrentFileDir)
	Else If WorldLevelName$(Level)="res10.dat"
		path$="res10.dat"
	Else
		path$="levels\"+WorldLevelName$(Level)
	EndIf
	WriteLine RecFile,"S "+RndSeed()+" "+DetailLevel+" "+DoesSoundExist+" "+RecBits(OptionsSoundLevel)+" "+NoTimeLimit+" "+ClassicDeathSound+" "+MenuStyle+" "+MenuBkg+" "+Language+" "+Level+" "+path$
End Function

; At the end of StartLevel.
Function RecLevelStarted()
	If RecOn=False Then Return
	WriteLine RecFile,"s "+RndSeed()+" "+MapMilliTime
	RecFrameMs=-1
End Function

; Before each UpdateGame: the keys as logic::Input packs them, plus two
; bits for the debug cheats the port does not model.
Function RecTickStart()
	If RecOn=False Then Return
	Local b=0
	RecSeedBefore=RndSeed()
	If KeyDown(200) Or KeyDown(72) Then b=b+1
	If KeyDown(208) Or KeyDown(80) Then b=b+2
	If KeyDown(203) Or KeyDown(75) Then b=b+4
	If KeyDown(205) Or KeyDown(77) Then b=b+8
	If KeyDown(42) Or KeyDown(54) Then b=b+16
	If KeyDown(28) Or KeyDown(156) Then b=b+32
	If KeyDown(56) And KeyDown(45) Then b=b+64	; Alt+X camera cheat
	If KeyDown(29) Or KeyDown(157) Then b=b+128	; Ctrl+F5/F6/F9
	RecInput=b
End Function

; After a tick. MainGameLoop calls it for the last tick of a frame only
; after the time-limit check, which the port runs at the end of each tick.
Function RecTickEnd()
	Local f=RecFile,s$,h,i,x,y
	If RecOn=False Then Return
	WriteLine f,"T "+LevelTimer+" "+RecSeedBefore+" "+RecInput+" "+RndSeed()

	WriteLine f,"W "+LevelCompleted+" "+LevelCompletedTimer+" "+MapTime+" "+BothCompleted+" "+FocusSwitchTimer+" "+SignActive+" "+ConveyorActive(1)+" "+ConveyorActive(2)+" "+View3dRotation+" "+View3dRotation2+" "+ShiftKeyPressed+" "+LightningTimer+" "+FinalGateAnim+" "+CurrentLightRed+" "+CurrentLightGreen+" "+CurrentLightBlue+" "+TargetLightRed+" "+TargetLightGreen+" "+TargetLightBlue+" "+LightChangeSpeed+" "+CurrentAmbientRed+" "+CurrentAmbientGreen+" "+CurrentAmbientBlue+" "+TargetAmbientRed+" "+TargetAmbientGreen+" "+TargetAmbientBlue+" "+AmbientChangeSpeed

	WriteLine f,"C "+RecBits(EntityX(Camera,True))+" "+RecBits(EntityY(Camera,True))+" "+RecBits(EntityZ(Camera,True))+" "+RecBits(EntityPitch(Camera,True))+" "+RecBits(EntityYaw(Camera,True))+" "+RecBits(FreeCameraX)+" "+RecBits(FreeCameraZ)+" "+RecBits(CameraDX)+" "+RecBits(CameraDZ)

	For i=1 To 4
		s$="P "+i+" "+PlayerExists(i)+" "+PlayerX(i)+" "+PlayerY(i)+" "+PlayerX2(i)+" "+PlayerY2(i)+" "+PlayerMoving(i)+" "+PlayerMovingTimer(i)+" "+PlayerRotation(i)+" "+PlayerDying(i)+" "+PlayerDyingTimer(i)+" "+PlayerDead(i)+" "+FocusOnPlayer(i)+" "+PlayerShrink(i)+" "+PlayerAnimCurrent(i)+" "+PlayerJumping(i)
		If PlayerExists(i)
			s$=s$+" "+RecBits(EntityX(PlayerEntity(i)))+" "+RecBits(EntityY(PlayerEntity(i)))+" "+RecBits(EntityZ(PlayerEntity(i)))
		Else
			s$=s$+" 0 0 0"
		EndIf
		WriteLine f,s$
	Next

	s$="B "+NofBoxes
	For i=0 To NofBoxes-1
		s$=s$+" "+BoxType(i)+" "+BoxX(i)+" "+BoxY(i)+" "+BoxMoving(i)+" "+BoxMovingTimer(i)+" "+BoxDead(i)+" "+RecBits(EntityX(BoxEntity(i)))+" "+RecBits(EntityY(BoxEntity(i)))+" "+RecBits(EntityZ(BoxEntity(i)))
	Next
	WriteLine f,s$

	s$="X "+NofBoxers
	For i=0 To NofBoxers-1
		s$=s$+" "+BoxerX(i)+" "+BoxerY(i)+" "+BoxerType(i)+" "+BoxerTimer(i)
	Next
	s$=s$+" "+NofTransporters
	For i=0 To NofTransporters-1
		s$=s$+" "+TransporterX(i)+" "+TransporterY(i)+" "+TransporterMoving(i)+" "+TransporterMovingTimer(i)
	Next
	WriteLine f,s$

	s$="M "+NofMonsters
	For i=0 To NofMonsters-1
		s$=s$+" "+MonsterType(i)+" "+MonsterX(i)+" "+MonsterY(i)+" "+MonsterMoving(i)+" "+MonsterMovingTimer(i)*MonsterMoving(i)+" "+MonsterDying(i)+" "+MonsterDead(i)
	Next
	WriteLine f,s$

	s$="I "+NofCrystals+" "+NofBonusCoins+" "+CurrentLevelScore+" "+NofItems
	For i=0 To NofItems-1
		s$=s$+" "+ItemType(i)+" "+ItemX(i)+" "+ItemY(i)
	Next
	WriteLine f,s$

	s$="K "+NofButtons
	For i=0 To NofButtons-1
		s$=s$+" "+ButtonType(i)+" "+ButtonActive(i)+" "+ButtonTimer(i)
	Next
	For i=0 To 9
		s$=s$+" "+LaserGateActive(i)
	Next
	WriteLine f,s$

	s$="R"
	For i=0 To 3
		s$=s$+" "+ElectroTimer(i)
	Next
	s$=s$+" "+NofSpikes
	For i=0 To NofSpikes-1
		s$=s$+" "+SpikeTimer(i)
	Next
	s$=s$+" "+NofCannons
	For i=0 To NofCannons-1
		s$=s$+" "+CannonTimer(i)
	Next
	s$=s$+" "+NofFireballs
	For i=0 To NofFireballs-1
		s$=s$+" "+FireballX(i)+" "+FireballY(i)+" "+FireballDead(i)
	Next
	WriteLine f,s$

	; MapTile hash: h = h * 31 + tile, row by row, wrapping at 32 bits.
	For y=0 To MapHeight-1
		For x=0 To MapWidth-1
			h=h*31+MapTile(x,y)
		Next
	Next
	WriteLine f,"G "+h
End Function

; After MainGameLoop's time-limit check.
Function RecFrame(ticks)
	If RecOn=False Then Return
	RecTickEnd()
	WriteLine RecFile,"F "+ticks+" "+RecFrameMs
	RecFrameMs=-1
End Function
