
Function MenuSetup()
	
	



	

	WorldMap=CreateMesh()
		i=0
		Repeat
		
			
		
			If worldmaptexturename$(i)<>"END"
				If version>=10 Or i=0 Or i=4
					WorldMapTexture(i)=MyLoadTexture("graphics\"+worldmaptexturename$(i),1,False)
				Else
					WorldMapTexture(i)=MyLoadTexture("graphics\"+Left$(worldmaptexturename$(i),Len(worldmaptexturename$(i))-4)+".jpg",1,False)

				EndIf
				
				Brush=CreateBrush()
				BrushTexture Brush,WorldMapTexture(i)
				WorldMapSurface(i)=CreateSurface(Worldmap,Brush)
				AddVertex(WorldMapSurface(i),0+WorldmapSquareX(i)*2,0,-2-WorldmapSquareY(i)*2-.001,0.001,.999)
				AddVertex(WorldMapSurface(i),2+WorldmapSquareX(i)*2+.001,0,-2-WorldmapSquareY(i)*2-.001,.999,.999)
				AddVertex(WorldMapSurface(i),2+WorldmapSquareX(i)*2+.001,0,0-WorldmapSquareY(i)*2,.999,0.001)
				AddVertex(WorldMapSurface(i),0+WorldmapSquareX(i)*2,0,0-WorldmapSquareY(i)*2,0.001,0.001)
				AddTriangle(WorldMapSurface(i),2,1,0)
				AddTriangle(WorldMapSurface(i),3,2,0)
				
				FreeBrush Brush
				
				
				
			EndIf
			i=i+1
		Until Worldmaptexturename$(i-1)="END"
		
		UpdateNormals(WorldMap)
		HideEntity Worldmap
		
		
	StoryScreen=CreateMesh()
		
		
		; just a single texture in the centre
		surface=CreateSurface(Storyscreen)
		AddVertex(surface,0+0.7,0,-2.6-0.05-.001,0.01,.99)
		AddVertex(surface,2+1.3+.001,0,-2.6-0.05-.01,.99,.99)
		AddVertex(surface,2+1.3+.001,0,0-0.05,.99,0.01)
		AddVertex(surface,0+0.7,0,0-0.05,0.01,0.01)
		AddTriangle(surface,2,1,0)
		AddTriangle(surface,3,2,0)
		UpdateNormals(Storyscreen)

		HideEntity StoryScreen

	CameraFogMode Camera,1
	CameraFogRange Camera,900,1000
	CameraFogColor Camera,0,0,0;128,175,255
	
	For i=1 To 11
		Menutexture(i)=MyLoadTexture("graphics\title"+Str$(i)+".bmp",4,False)
		MenuEntity(i)=CreateCube(camera);Sprite(camera)
		ScaleEntity MenuEntity(i),1,1,0.01
		EntityTexture MenuEntity(i),MenuTexture(i)
		PositionEntity MenuEntity(i),-11+2*i,5,24
	Next
	
	MenuEntity(12)=CreateSphere(32)
	RotateMesh MenuEntity(12),90,0,0
	ScaleEntity MenuEntity(12),13,13,500
	;PositionEntity MenuEntity(12),0,0,495
	
	MenuTexture(12)=myloadtexture("graphics\rainbow3.bmp",1,False)	
	FlipMesh MenuEntity(12)
	EntityTexture MenuEntity(12),menutexture(12)
	
			
	MenuSelector = CreateCube (Camera)
	MenuSelectortex = MyLoadTexture("graphics\menubar.jpg",1,False)
	ScaleEntity MenuSelector,.35,.02,0.01
	TranslateEntity MenuSelector,-.5+20*.02+.01,0.375-(-2000)*.04-.02,0
	EntityTexture MenuSelector,MenuSelectortex
	EntityAlpha MenuSelector,.6
	
	Worldmapshade1 = CreateCube (Camera)
	ScaleEntity Worldmapshade1,.72,.08,0.01
	TranslateEntity Worldmapshade1,-.5+20*.02+.01,0.375-(-2000)*.04-.02,0
	EntityTexture Worldmapshade1,MenuSelectortex
	EntityAlpha Worldmapshade1,.6
	HideEntity Worldmapshade1
	
	Worldmapshade2 = CreateCube (Camera)
	ScaleEntity Worldmapshade2,.72,.10,0.01
	TranslateEntity Worldmapshade2,-.5+20*.02+.01,0.175-(-2000)*.04-.02,0
	EntityTexture Worldmapshade2,MenuSelectortex
	EntityAlpha Worldmapshade2,.6
	HideEntity Worldmapshade2
	

	
	WorldLevelTexture=MyLoadTexture("graphics\levelsigns2.bmp",4,False)
	WorldLevelTexture2=MyLoadTexture("graphics\levelsigns2b.bmp",4,False)
	
	
	
	If version>=10 
		For i=1 To Nofworldmaplevels-1
			If Upper$(Right$(worldmaplevelname$(i),4))=".LV5"
				;Print i
				File=MyReadFile("wswlevels\"+WorldMapLevelName$(i),False)
		
				ex$=ReadString$(File) 	
				ex$=ReadString$(File)
				ix=ReadInt(File)
				ex$=ReadString$(File)
				CloseFile(File)
				
								
				j=1
				k=0
				While(j<Len(ex$))
					If Mid$(ex$,j,1)="#" 
						CurrentWorldLevelName$(i,k)=Left$(ex$,j-1)
						k=k+1
						ex$=Right$(ex$,Len(ex$)-j)
						j=0
					EndIf
					j=j+1
				Wend
				CurrentWorldLevelName$(i,k)=ex$
				;Print ex$
				;Delay 1000
				If k<2 Then CurrentWorldLevelName$(i,2)=""
				If k<1 Then CurrentWorldLevelName$(i,2)=""
			EndIf
		Next
	Else
	For i=1 To 6
			If Upper$(Right$(worldmaplevelname$(i),4))=".LV5"
				;Print i
				File=MyReadFile("wswlevels\"+WorldMapLevelName$(i),False)
		
				ex$=ReadString$(File) 	
				ex$=ReadString$(File)
				ix=ReadInt(File)
				ex$=ReadString$(File)
				CloseFile(File)
				
								
				j=1
				k=0
				While(j<Len(ex$))
					If Mid$(ex$,j,1)="#" 
						CurrentWorldLevelName$(i,k)=Left$(ex$,j-1)
						k=k+1
						ex$=Right$(ex$,Len(ex$)-j)
						j=0
					EndIf
					j=j+1
				Wend
				CurrentWorldLevelName$(i,k)=ex$
				;Print ex$
				;Delay 1000
				If k<2 Then CurrentWorldLevelName$(i,2)=""
				If k<1 Then CurrentWorldLevelName$(i,2)=""
			EndIf
		Next
For i=126 To 131
			If Upper$(Right$(worldmaplevelname$(i),4))=".LV5"
				;Print i
				File=MyReadFile("wswlevels\"+WorldMapLevelName$(i),False)
		
				ex$=ReadString$(File) 	
				ex$=ReadString$(File)
				ix=ReadInt(File)
				ex$=ReadString$(File)
				CloseFile(File)
				
								
				j=1
				k=0
				While(j<Len(ex$))
					If Mid$(ex$,j,1)="#" 
						CurrentWorldLevelName$(i,k)=Left$(ex$,j-1)
						k=k+1
						ex$=Right$(ex$,Len(ex$)-j)
						j=0
					EndIf
					j=j+1
				Wend
				CurrentWorldLevelName$(i,k)=ex$
				;Print ex$
				;Delay 1000
				If k<2 Then CurrentWorldLevelName$(i,2)=""
				If k<1 Then CurrentWorldLevelName$(i,2)=""
			EndIf
		Next

	
	
	EndIf
	


End Function	




Function StartMenu(Menu)

	
	If doessoundexist=True And menu=1
		If currentmusic<>5
			StopChannel Music
			Music=PlayMusic("res7.dat")	
			ChannelVolume music,OptionsMusicLevel
			currentmusic=5
		EndIf
	EndIf


	CurrentMenuOption=0
	
	CameraRange Camera,.98,1000

	; General Stuff

	; Camera & Lights
	If Menu<>5 ;  no changes for in-game Menu
		RotateEntity Camera,0,0,0
		PositionEntity Camera,0,0,0
		
	;	PositionEntity LevelLight,0,0,-100
	;	RotateEntity LevelLight,25,0,0

		
	EndIf
	
	
	
	
	If Menu<>4 And Menu<>5	And Menu<>12 And Menu<>16 ShowEntity MenuSelector

	
	
	; Special Stuff	
	If Menu=1 	; Main Menu
	
		For i=1 To 12
			ShowEntity MenuEntity(i)
		Next
		
		
		HidePointer()
		

		
		

	Else If Menu=2	; Player Select Screen
			
		ShowEntity MenuEntity(12)		
		
					
		; Special Stuff: Collect Names
		; for the select player name, check first if there are any players at all
		; if not go directly to enter player name
		
		; but first check if that directory exitst
		If FileType("players")<>2 Then CreateDir("players")
		
		Dir=ReadDir("players")
		PlayerNameN=0
		; first count all files
		Repeat
			FileName$=NextFile$(Dir)
			If FileName$="" Then Exit
			If Upper$(Right$(FileName$,4))=".PLA" 
				PlayerName$(PlayerNameN)=Left$(FileName$,Len(FileName$)-4)
				PlayerNameN=PlayerNameN+1
			EndIf
			If PlayerNameN=10 Then Exit
		Forever
		
		CloseDir(Dir)

		; no players exist - go to create player
		If PlayerNameN=0 
		 	StartMenu(3)
			Return
		EndIf
		
		; Yes, there are players - get their Levelscompleted and Score
		For i=0 To PlayerNameN-1
		
			File=ReadFile("players/"+Lower$(PlayerName$(i))+".pla")
			ex$=ReadString$(File)
			If ex$<>"Stinky & Loof WSW Player File"
				; uh oh! old player file
				PlayerNameN=PlayerNameN-1
				
				CloseFile(File)
				CopyFile "players/"+Lower$(PlayerName$(i))+".pla","players/"+Lower$(PlayerName$(i))+".old"
				DeleteFile "players/"+Lower$(PlayerName$(i))+".pla"
				i=i-1
				If PlayerNameN=0
					StartMenu(3)
					Return
				EndIf
			Else
				ex$=ReadString$(File) ; name
				ix=ReadInt(File) ; ID
				ix=ReadInt(File) ; world
				PlayerLevels(i)=ReadInt(File)
				PLayerScore(i)=ReadInt(File)
				
				
			
	
				
				
				CloseFile(File)
			EndIf
			
		Next

	
	Else If Menu=3; Enter New Player Name
		ShowEntity MenuEntity(12)
		; Music
		
		
		; Special Stuff
		CurrentPlayerName$=""	
	
	Else If Menu=4; Level Selector
	
		; Start Map-Screen
		;If DoesSoundExist=True And music>0 And (currentmenu=2 Or currentmenu=3)
		;	FreeSound music
		;	music=0
		;EndIf
		StartMapScreen()
	

		
		
		

	
	Else If Menu=5; In Game Menu
		; Camera - No Change
		
		; Signs
		StopSign(2)
		For i=11 To 30
			StopSign(i)
		Next
		
		; Lights
		LightColor LevelLight,0,0,0
		AmbientLight 100,100,100
		
		
		; LevelTimer - No Change
		
	
	Else If menu=6 Or menu=9 Or menu=10 Or menu=15 Or menu=18 
		ShowEntity MenuEntity(12)	
	
		
	Else If menu=11
		ShowEntity MenuEntity(12)
		leveltimer=0	
		; load demo pics
		For i=0 To 5
			Teleporttexture(i)=MyLoadTexture("graphics\demo"+Str$(i+1)+".jpg",1,False)
			BoxEntity(i)=CreateFlatSquare(TeleportTexture(i))
			RotateEntity boxentity(i),-90,0,0
		;	RotateEntity boxentity(i),0,0,-25+i*10
			
		Next
		
		
	

		
	Else If Menu=16	; End of Game Deluxe
		
		LevelTimer=0
	;	MenuTimer=12000
		
		;RotateEntity Camera,90,0,0
		;HideEntity MenuEntity(4)
		
		
	;	CameraClsColor Camera,170,170,255
	;	CameraFogColor Camera,170,170,255
		;CameraRange Camera,.98,10000
		;CameraFogRange Camera,1000,10000
		;CameraFogMode Camera,1
		
		PositionEntity Camera,0,3,-10
		RotateEntity Camera,10,0,0

		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255

		
		
		; Music
		If DoesSoundExist=True; And CurrentMenu<>7 And CurrentMenu<>9 And CurrentMenu<>10
			StopChannel(Music)
			Music=PlayMusic("res7.dat")
			ChannelVolume music,OptionsMusicLevel
		EndIf

	Else If menu=19
		ShowEntity MenuEntity(12)
		leveltimer=0		


	EndIf
	
	CurrentMenu=Menu
	
	GamePhase=0
		
	TweenPeriod=1000/85
	TweenTime=MilliSecs()-TweenPeriod


End Function

Function MenuLoop()
	
;	If CurrentMenu<>5 Then LevelTimer=LevelTimer+1

	
	If CurrentMenu=1 Then MenuLoop1			;main menu
	If CurrentMenu=2 Then MenuLoop2			;select player
	If CurrentMenu=3 Then MenuLoop3			;new player
	If CurrentMenu=4 Then MapScreenLoop		;map screen mode
	If CurrentMenu=5 Then MenuLoop5			;in-game menu
	If CurrentMenu=6 Then MenuLoop6			;confirm player deletion
	If CurrentMenu=7 Then StoryScreenLoop	;story screen mode

	If CurrentMenu=9 Then MenuLoop9			;options
	If CurrentMenu=10 Then MenuLoop10		;exit to windows
	If CurrentMenu=11 Then MenuLoop11		;demo ad
	If CurrentMenu=15 Then MenuLoop15		;go online confirmation
	If CurrentMenu=16 Then endingupdate		;end of game
	If CurrentMenu=18 Then MenuLoop18		;story screens
	If CurrentMenu=19 Then MenuLoop19		;credits



	If KeyHit(66) Then
		SaveBuffer(FrontBuffer(),"screenshot.bmp")
	End If


	
End Function


Function MenuLoop1()
	
	
	; Display 
	centertext("",15,1,54)
	centertext("",15,1,55)
	centertext("",15,1,58)
	centertext("",15,1,59)
	centertext("",15,1,60)
	DisplayText("v"+Str$(VersionNo1)+"."+Str$(VersionNo2)+Str$(VersionNo3),44,0,1,50)
	If Version<10 Then DisplayText("Demo",46,1,1,54)
	If Language=0
		;CenterText("MAIN MENU",10,1,55)
		CenterText("START GAME",10,1,51)
		CenterText("OPTIONS",11,1,52)
		CenterText("ONLINE SOLUTIONS",12,1,53)
		;DisplayText("LEVEL EXCHANGE",12,1,54)
		CenterText("STORY PICTURES",13,1,57)
		CenterText("CREDITS",14,1,58)
		
		If Version<10 
			CenterText("GET FULL VERSION",15,1,55)
		Else
			centerText("DISCUSSION BOARD",15,1,55)
		EndIf
		CenterText("EXIT TO WINDOWS",16,1,56)
	Else If Language=1
		CenterText("SPIEL STARTEN",9,1,51)
		CenterText("OPTIONEN",10,1,52)
		CenterText("ONLINE HILFE",11,1,53)
		CenterText("ONLINE FORUM",12,1,54)
		CenterText("LEVEL EDITOR",13,1,55)
		CenterText("SPIEL BEENDEN",14,1,56)
	Else If Language=2
		CenterText("JOUER",9,1,51)
		CenterText("OPTIONS",10,1,52)
		CenterText("SOLUCES EN LIGNE",11,1,53)
		CenterText("FORUM EN LIGNE",12,1,54)
		CenterText("LEVEL EDITOR",13,1,55)
		CenterText("SORTIE VERS WINDOWS",14,1,56)
	EndIf
	; Use Cursor Keys
	If MenuTimer Mod 300 <150
		If Language=0
			CenterText("Use Cursor Keys and 'Enter' to select an option.",18,1,59)
			

		EndIf
	
	Else 

		CenterText("",17,1,59)
	EndIf
	
	For i=1 To 10
		PositionEntity MenuEntity(i),-11+2*i+.7*Sin(menutimer/2+i*36),5+.7*Cos(menutimer/2+i*36),24+3*Sin(menutimer/4+i*36)
	Next
	PositionEntity MenuEntity(11),0,0.3,4+Cos(menutimer/6)
	
	PositionTexture menutexture(12),0,watertexv
	RotateEntity menuentity(12),0,0,menutimer/10 Mod 360

	
	; General Update Routine	
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(10+CurrentMenuOption)*.044+.027,1.1
	ScaleEntity MenuSelector,48.5*.012,.02,0.01
	
	

	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next
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
	
		; do the updates in here
		MenuBkgUpdate()
		

		UpdateWorld
	
	Next

	RenderWorld tween
	Flip
	

	; Input Etc
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)		
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=6
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption+1
			If CurrentMenuOption=7 Then CurrentMenuOption=0
			CursorKeyPressed=True
		EndIf
	
	Else If (KeyDown(28) Or KeyDown(156))
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CursorKeyPressed=True
			If CurrentMenuOption=0
				EndMenu()
				StartMenu(2)
				Return
			EndIf
			If CurrentMenuOption=1
				EndMenu()
				StartMenu(9)
				Return
			EndIf
			If CurrentMenuOption=2
				EndMenu()
				VersionNo1=-999
				StartMenu(15)
			EndIf
			If CurrentMenuOption=3
				EndMenu()
				StartMenu(18)
				Return
			EndIf
			If CurrentMenuOption=4
				EndMenu()
				StartMenu(19)
				Return

			EndIf
			If CurrentMenuOption=5
				EndMenu()
				If Version<10
					VersionNo1=-998
				Else
					VersionNo1=-997
				EndIf
				StartMenu(15)
			EndIf
			If CurrentMenuOption=6
				EndMenu()
				StartMenu(10)

								
			EndIf
			
			
		EndIf
	
	Else If KeyDown(1)
		If CursorKeyPressed=False
			CursorKeyPressed=True
			EndMenu()
			StartMenu(10)
			

		EndIf
	Else
		CursorKeyPressed=False
	EndIf
	
		

End Function

Function MenuLoop2()
	

	; Display Static Text
	If Language=0
		CenterText( "SELECT PLAYER",4,1,50)
		CenterText( "----------------------------------------------",5,1,51)
		DisplayText("Name                 Levels        Total Score",2,6,1,52)
		
		DisplayText("CREATE NEW PLAYER",2,7,1,53)
	Else If Language=1
		CenterText("SPIELERAUSWAHL",4,1,50)
		CenterText("------------------------------",5,1,51)
		DisplayText("Name             Levels Punkte",10,6,1,52)
		
		DisplayText("NEUER SPIELER",10,7,1,53)
	Else If Language=2
		CenterText("CHOISIS LE JOUEUR",4,1,50)
		CenterText("------------------------------",5,1,51)
		DisplayText("Nom              Niveau  Score",10,6,1,52)
		
		DisplayText("CREER NOUVEAU JOUEUR",10,7,1,53)
	EndIf
	If PlayerNameN>0
		For i=0 To PlayerNameN-1
			DisplayText(Upper$(PlayerName$(i)),2,8+i,1,54+4*i)
			DisplayText(PlayerLevels(i),29-Len(Str$(PlayerLevels(i))),8+i,1,55+4*i)
			;DisplayText(PlayerExtraLevels(i),33-Len(Str$(PlayerExtraLevels(i))),8+i,1,57+4*i)
			DisplayText(PlayerScore(i),48-Len(Str$(PlayerScore(i))),8+i,1,56+4*i)
		Next 
	EndIf
	CenterText("----------------------------------------------",8+PlayerNameN,1,55+4*PlayerNameN)

	
		
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(7+CurrentMenuOption)*.044+.027,1.1
	ScaleEntity MenuSelector,48.5*.012,.02,0.01
	

	For i=1 To 10
		PositionEntity MenuEntity(i),-11+2*i+.7*Sin(menutimer+i*36),5+.7*Cos(menutimer+i*36),24+3*Sin(menutimer/2+i*36)
	Next
	PositionEntity MenuEntity(11),0,0.3,13+10*Cos(menutimer/6)
	
	PositionTexture menutexture(12),0,watertexv
	RotateEntity menuentity(12),0,0,menutimer/10 Mod 360


	
	
	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next



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

		
		; do the updates in here
		MenuBkgUpdate()		
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	
	
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False		
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=PlayerNameN
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption+1
			If CurrentMenuOption=PlayerNameN+1 Then CurrentMenuOption=0
			CursorKeyPressed=True
		EndIf
	Else If (KeyDown(28) Or KeyDown(156))
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CursorKeyPressed=True
			If CurrentMenuOption=0 
				; create new player
				If PlayerNameN<10
					EndMenu()
					StartMenu(3)
					Return
				Else
					; no more room - maybe play a sound fx here?
				EndIf
			Else
				; select existing player
				EndMenu()
				LoadPlayer(PlayerName$(CurrentMenuOption-1))
				StartMenu(4)				
				Return
			EndIf
		EndIf
	Else If KeyDown(211)
		If CursorKeyPressed=False
			CursorKeyPressed=True
			If CurrentMenuOption>0
				Player2BDeleted=CurrentMenuOption-1
				EndMenu()
				StartMenu(6)
				Return
			EndIf
		EndIf
		
	Else If KeyDown(1)
		If CursorKeyPressed=False
			CursorKeyPressed=True
			EndMenu()
			StartMenu(1)
			Return
		EndIf
				
	Else
		CursorKeyPressed=False
	EndIf
	
			
			 

End Function

Function MenuLoop3()
	

	; Display Static Text
	If Language=0
		CenterText("NEW PLAYER - PLEASE ENTER NAME:",4,1,50)
		CenterText("-------------------------------",5,1,51)
	Else If Language=1
		CenterText("SPIELER - BITTE NAMEN EINGEBEN:",4,1,50)
		CenterText("-------------------------------",5,1,51)
	Else If Language=2
		CenterText("NOUVEAU JOUEUR - S.V.P. ECRIS TON NOM:",4,1,50)
		CenterText("--------------------------------------",5,1,51)

	EndIf
	
	CenterText(CurrentPlayerName$,7,1,52)
	If LevelTimer Mod 50<25
		DisplayText(":",25+Len(CurrentPlayerName$)/2.0,7,1,53)
	Else
		DisplayText(" ",25+Len(CurrentPlayerName$)/2.0,7,1,53)
	EndIf
	
	
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.375-Float(-2)*.04-.013,1.001
		PositionTexture menutexture(12),0,watertexv
	RotateEntity menuentity(12),0,0,menutimer/10 Mod 360
		
	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next



	
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

		
		; do the updates in here
		
		MenuBkgUpdate()		
		
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip
	
	;ascii keys
	askey=GetKey()
	If askey>=97 Then askey=askey-32
	If (askey>=65 And askey<=90) And Len(CurrentPlayerName$)<15
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(1),0)	
			CurrentPlayerName$=CurrentPlayerName$+Chr$(askey)
			CursorKeyPressed=True
		EndIf
	Else If (askey=32) And Len(CurrentPlayerName$)<15
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(1),0)	
			CurrentPlayerName$=CurrentPlayerName$+" "
			CursorKeyPressed=True
		EndIf

		
		
	; backspace
	Else If KeyDown(14) And Len(CurrentPlayerName$)>0
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(1),0)	
			CurrentPlayerName$=Left$(CurrentPlayerName$,Len(CurrentPlayerName$)-1)
			CursorKeyPressed=True
		EndIf
	
	; enter	
	Else If (KeyDown(28) Or KeyDown(156))
		If CursorKeyPressed=False
			PlayMySound(SFXLastCoin,0)	
		
			CreateNewPlayer()
			
			CursorKeyPressed=True
			EndMenu()
			
			StartStoryScreen(1)
			Return
		EndIf
	Else If KeyDown(1)
		If CursorKeyPressed=False
			CursorKeyPressed=True
			EndMenu()
			StartMenu(2)
			Return
		EndIf
	
	Else
		CursorKeyPressed=False
	EndIf
	
	

		
			 

End Function



Function MenuLoop5()

	
	If Language=0
	
		If CurrentMenuOption=0
			CenterText("== RESTART THIS LEVEL ==",7,1,52)
		Else
			CenterText("RESTART THIS LEVEL",7,1,52)
		EndIf
		
		If CurrentMenuOption=1
			CenterText("== SELECT NEW LEVEL ==",8,1,53)
		Else
			CenterText("SELECT NEW LEVEL",8,1,53)
		EndIf
		
		If CurrentMenuOption=2
			CenterText("== EXIT GAME ==",9,1,54)
		Else
			CenterText("EXIT GAME",9,1,54)
		EndIf
		
		If CurrentMenuOption=3
			CenterText("== RESUME GAME ==",10,1,55)
		Else
			CenterText("RESUME GAME",10,1,55)
		EndIf
		
	;	If CurrentMenuOption=4
	;		CenterText("== SAVE GAME STATE ==",12,1,56)
	;	Else
	;		CenterText("SAVE GAME STATE",12,1,56)
	;	EndIf
	;	If SaveGameStateActive=True
	;		If CurrentMenuOption=5
	;			CenterText("== LOAD GAME STATE ==",13,1,57)
	;		Else
	;			CenterText("LOAD GAME STATE",13,1,57)
	;		EndIf
	;	EndIf


	Else If Language=1
		If CurrentMenuOption=0
			CenterText("== LEVEL NEU STARTEN ==",7,1,52)
		Else
			CenterText("LEVEL NEU STARTEN",7,1,52)
		EndIf
		
		If CurrentMenuOption=1
			CenterText("== NEUES LEVEL ==",8,1,53)
		Else
			CenterText("NEUES LEVEL",8,1,53)
		EndIf
		
		If CurrentMenuOption=2
			CenterText("== SPIEL BEENDEN ==",9,1,54)
		Else
			CenterText("SPIEL BEENDEN",9,1,54)
		EndIf
		
		If CurrentMenuOption=3
			CenterText("== SPIEL FORTSETZEN ==",10,1,55)
		Else
			CenterText("SPIEL FORTSETZEN",10,1,55)
		EndIf
		
	Else If Language=2
		If CurrentMenuOption=0
			CenterText("== RECOMMENCER CE NIVEAU ==",7,1,52)
		Else
			CenterText("RECOMMENCER CE NIVEAU",7,1,52)
		EndIf
		
		If CurrentMenuOption=1
			CenterText("== CHOISIR UN AUTRE NIVEAU ==",8,1,53)
		Else
			CenterText("CHOISIR UN AUTRE NIVEAU",8,1,53)
		EndIf
		
		If CurrentMenuOption=2
			CenterText("== SORTIE DU JEU ==",9,1,54)
		Else
			CenterText("SORTIE DU JEU",9,1,54)
		EndIf
		
		
			If CurrentMenuOption=3
				CenterText("== CONTINUER A JOUER ==",10,1,55)
			Else
				CenterText("CONTINUER A JOUER",10,1,55)
			EndIf
		
	EndIf

	
	
	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next




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
	
		
		;UpdateWorld
	
	Next
		
	RenderWorld tween
	
;	Color 255,255,255 : Text 0,DisplayHeight-48,"Total Videomemory : " + TotalVidMem() + " - Available Video Memory : " + AvailVidMem()
;Text 0,DisplayHeight-32,"Active Textures : "+ActiveTextures()
;Text 0,DisplayHeight-16,"Tris Rendered : "+TrisRendered()
	
	Flip
	
	
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False	
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=3;4+SaveGameStateActive
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)
			CurrentMenuOption=CurrentMenuOption+1
			If CurrentMenuOption=4 Then CurrentMenuOption=0
			CursorKeyPressed=True
		EndIf
	Else If (KeyDown(28) Or KeyDown(156))
	
		While (KeyDown(28) Or KeyDown(156))
		Wend
		
	
		; Clear This Menu
		For l.LetterType=Each LetterType
			FreeEntity l\Entity
			Delete l
		Next
		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255
		
		If CurrentMenuOption=0
			For k=1 To 4
				If PlayerExists(k)=True Then KillPlayer(k)
			Next
			GamePhase=1
		Else If CurrentMenuOption=1
			; Back to Menu
			EndLevel()
			
				If levelcompleted=1 
					If currentlevel=6 And storytold(0)=0 ; meet loof
						storytold(0)=1
						startstoryscreen(6)	;DITTO IN IN UPDATEGAME!!!!!!!!!!!!!!!!!!!!!
					Else If currentlevel=13 And storytold(1)=0 ; meet peegue
						storytold(1)=1
						startstoryscreen(10)
					Else If currentlevel=30 And storytold(2)=0 ; meet peegue
						storytold(2)=1
						startstoryscreen(24)
					Else If currentlevel=116
						
						complete=0
						perfect=0
						For i=1 To NofWorldMapLevels-1
							If WorldMapLevelStatus(i)=1
								complete=complete+1
							Else If WorldMapLevelStatus(i)=2
								complete=complete+1
								perfect=perfect+1
							EndIf
						Next
						
						If complete=109 And perfect=109
							startstoryscreen(46)
						Else If complete=109
							startstoryscreen(45)
						Else
							startstoryscreen(44)
						EndIf



					Else
						StartMenu(4)
					EndIf

					
				Else
					StartMenu(4)
				EndIf
			
			
		Else If CurrentMenuOption=2
			; Exit Game
			EndLevel()
			StartMenu(1)
		
		Else If CurrentMenuOption=3
			; Resume Game
					
			GamePhase=1
			CheckSigns()
			
		Else If CurrentMenuOption=4
			SaveGameState()
			GamePhase=1
			CheckSigns()
		Else If CurrentMenuOption=5
			LoadGameState()
			GamePhase=1
			CheckSigns()
	
		EndIf

	Else If KeyDown(1)
	
		While KeyDown(1)
		Wend
		
		; Esc: Back to Menu
		For l.LetterType=Each LetterType
			FreeEntity l\Entity
			Delete l
		Next
		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255
		
		
		GamePhase=1
		CheckSigns()

		
	Else
		CursorKeyPressed=False
	EndIf

		
	

End Function
Function MenuLoop6()

	; Display Static Text
	If Language=0
		CenterText("CONFIRM PLAYER DELETION?",4,1,50)
		CenterText("------------------------",5,1,51)
		CenterText("(backup is stored in PLAYER directory)",6,1,52)
		
		CenterText("YES",8,1,53)
		CenterText("NO",9,1,54)
	Else If Language=1
		CenterText("SPIELER LOESCHEN?",4,1,50)
		CenterText("-----------------",5,1,51)
		CenterText("(Backup im Player Directory)",6,1,52)
		
		CenterText("Ja",8,1,53)
		CenterText("Nein",9,1,54)
	Else If Language=2
		CenterText("CONFIRMEZ L'EFFACEMENT DU JOUER?",4,1,50)
		CenterText("-------------------------------",5,1,51)
		CenterText("La sauvegarde est stockee dans le dossier PLAYERS",6,1,52)
		
		CenterText("OUI",8,1,53)
		CenterText("NON",9,1,54)

	EndIf
	
	
	
		
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(8+CurrentMenuOption)*.044+.027,1.1
	ScaleEntity MenuSelector,4.5*.012,.02,0.01
	

		PositionTexture menutexture(12),0,watertexv
	RotateEntity menuentity(12),0,0,menutimer/10 Mod 360
	
	
	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next



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

		
		; do the updates in here
		
		LevelTimer=LevelTimer+1
	
		
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	
	
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False		
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=1
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption+1
			If CurrentMenuOption=2 Then CurrentMenuOption=0
			CursorKeyPressed=True
		EndIf
	Else If (KeyDown(28) Or KeyDown(156))
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CursorKeyPressed=True
			If CurrentMenuOption=0 
				; delete
				CopyFile "players/"+PlayerName$(Player2BDeleted)+".pla","players/"+PlayerName$(Player2BDeleted)+".bak"
				DeleteFile "players/"+PlayerName$(Player2BDeleted)+".pla"
			EndIf
			EndMenu()
			StartMenu(2)
		EndIf
	Else
		CursorKeyPressed=False
	EndIf


	
End Function


Function MenuLoop9()

	If Language=0	
		If OptionsSoundLevel=1
			OptionsSoundText$="Sound Volume: High"
		Else If OptionsSoundLevel=.6
			OptionsSoundText$="Sound Volume: Medium"
		Else If OptionsSoundLevel=.2
			OptionsSoundText$="Sound Volume: Low"
		Else If OptionsSoundLevel=0
			OptionsSoundText$="No Sound"
		EndIf
		If OptionsMusicLevel=1
			OptionsMusicText$="Music Volume: High"
		Else If OptionsMusicLevel=.7
			OptionsMusicText$="Music Volume: Medium"
		Else If OptionsMusicLevel=.4
			OptionsMusicText$="Music Volume: Low"
		Else If OptionsMusicLevel=0
			OptionsMusicText$="No Music"
		EndIf
		OptionsResText$="Resolution: "+Str$(MyGfxModeWidth(MyGfxMode))+"x"+Str$(MyGfxModeHeight(MyGfxMode))
		If DetailLevel=5
			OptionsDetailText$="Detail Level: High"
		Else If DetailLevel=3
			OptionsDetailText$="Detail Level: Medium"
		Else If DetailLevel=1
			OptionsDetailText$="Detail Level: Low"
		EndIf
		If MyWindowed=0
			OptionsWindowedText$="Full Screen"
		Else
			OptionsWindowedText$="Windowed"
		EndIf
		If NoTimeLimit=1
			OptionsTimeLimitText$="No Time Limit"
		Else
			OptionsTimeLimitText$="Timed Mode"
		EndIf
		If LockWorlds=0
			LockWorldsText$="Normal World Progression"
		Else
			LockWorldsText$="All Worlds Unlocked"
		EndIf
	
		OptionsLanguageText$="English"
		
		If OptionsTeleport=0
			TeleportText$="Teleport: Pent"
		Else
			TeleportText$="Teleport: Star"
		EndIf
		
		If OPtionsSwitchPlayerSound=1
			SwitchText$="Switch SFX: New"
		Else
			SwitchText$="Switch SFX: Classic"
		EndIf
		

		
	
	
		; Options Menu
		
			
		CenterText("OPTIONS",6,1,50)
		CenterText("-------",7,1,51)
		CenterText(OptionsSoundText$,9,1,52)
		CenterText(OptionsMusicText$,10,1,53)
		;CenterText(OptionsResText$,10,1,54)
		CenterText(OptionsDetailText$,11,1,55)
		;CenterText(OptionsWindowedText$,12,1,56)
		;CenterText(OPtionsLanguageText$,13,1,58)
		CenterText(OPtionsTimeLimitText$,12,1,60)
		;CenterText(LockWorldsText$,12,1,54)
		;CenterText(TeleportText$,13,1,56)
		;CenterText(SwitchText$,14,1,61)

		CenterText("Visit Us Online",13,1,59)
		CenterText("MAIN MENU",14,1,57)
		
	Else If Language=1
	
		OptionsLanguageText$="Deutsch"
	
		If OptionsSoundLevel=1
			OptionsSoundText$="Sound: Laut"
		Else If OptionsSoundLevel=.6
			OptionsSoundText$="Sound: Mittel"
		Else If OptionsSoundLevel=.2
			OptionsSoundText$="Sound: Leise"
		Else If OptionsSoundLevel=0
			OptionsSoundText$="Kein Sound"
		EndIf
		If OptionsMusicLevel=1
			OptionsMusicText$="Musik: Laut"
		Else If OptionsMusicLevel=.7
			OptionsMusicText$="Musik: Mittel"
		Else If OptionsMusicLevel=.4
			OptionsMusicText$="Musik: Leise"
		Else If OptionsMusicLevel=0
			OptionsMusicText$="Keine Musik"
		EndIf
		OptionsResText$="Aufloesung: "+Str$(MyGfxModeWidth(MyGfxMode))+"x"+Str$(MyGfxModeHeight(MyGfxMode))
		If DetailLevel=5
			OptionsDetailText$="Detaillevel: Hoch"
		Else If DetailLevel=3
			OptionsDetailText$="Detaillevel: Mittel"
		Else If DetailLevel=1
			OptionsDetailText$="Detaillevel: Niedrig"
		EndIf
		If MyWindowed=0
			OptionsWindowedText$="Vollbild"
		Else
			OptionsWindowedText$="Fenster"
		EndIf
		If NoTimeLimit=1
			OptionsTimeLimitText$="Kein Zeitlimit"
		Else
			OptionsTimeLimitText$="Timer Modus"
		EndIf

	
	
	
		; Options Menu
		
			
		CenterText("OPTIONEN",5,1,50)
		CenterText("--------",6,1,51)
		CenterText(OptionsSoundText$,8,1,52)
		CenterText(OptionsMusicText$,9,1,53)
		CenterText(OptionsResText$,10,1,54)
		CenterText(OptionsDetailText$,11,1,55)
		CenterText(OptionsWindowedText$,12,1,56)
		CenterText(OPtionsLanguageText$,13,1,58)
		CenterText(OPtionsTimeLimitText$,14,1,60)

		CenterText("Unsere Webseite",15,1,59)
		CenterText("HAUPTMENUE",16,1,57)
	Else If Language=2
	
		OptionsLanguageText$="Francais"
	
		If OptionsSoundLevel=1
			OptionsSoundText$="Niveau du Son: 3"
		Else If OptionsSoundLevel=.6
			OptionsSoundText$="Niveau du Son: 2"
		Else If OptionsSoundLevel=.2
			OptionsSoundText$="Niveau du Son: 1"
		Else If OptionsSoundLevel=0
			OptionsSoundText$="Niveau du Son: 0"
		EndIf
		If OptionsMusicLevel=1
			OptionsMusicText$="Niveau de la Musique: 3"
		Else If OptionsMusicLevel=.7
			OptionsMusicText$="Niveau de la Musique: 2"
		Else If OptionsMusicLevel=.4
			OptionsMusicText$="Niveau de la Musique: 1"
		Else If OptionsMusicLevel=0
			OptionsMusicText$="Niveau de la Musique: 0"
		EndIf
		OptionsResText$="Resolution: "+Str$(MyGfxModeWidth(MyGfxMode))+"x"+Str$(MyGfxModeHeight(MyGfxMode))
		If DetailLevel=5
			OptionsDetailText$="Detail: Haut"
		Else If DetailLevel=3
			OptionsDetailText$="Detail: Medium"
		Else If DetailLevel=1
			OptionsDetailText$="Detail: Bas"
		EndIf
		If MyWindowed=0
			OptionsWindowedText$="Plein Ecran"
		Else
			OptionsWindowedText$="En Fenetre"
		EndIf
		If NoTimeLimit=1
			OptionsTimeLimitText$="Pas de Chrono"
		Else
			OptionsTimeLimitText$="Mode Chrono"
		EndIf

	
	
	
		; Options Menu
		
			
		CenterText("OPTIONS",5,1,50)
		CenterText("-------",6,1,51)
		CenterText(OptionsSoundText$,8,1,52)
		CenterText(OptionsMusicText$,9,1,53)
		CenterText(OptionsResText$,10,1,54)
		CenterText(OptionsDetailText$,11,1,55)
		CenterText(OptionsWindowedText$,12,1,56)
		CenterText(OPtionsLanguageText$,13,1,58)
		CenterText(OPtionsTimeLimitText$,14,1,60)
		CenterText("Notre Site Web",15,1,59)
		CenterText("Menu Principal",16,1,57)

	EndIf
	
	
		
	
		
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(9+CurrentMenuOption)*.044+.027,1.1
	ScaleEntity MenuSelector,22*.012,.02,0.01
	
	PositionTexture menutexture(12),0,watertexv
	RotateEntity menuentity(12),0,0,menutimer/10 Mod 360
	
	
	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next



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

		
		; do the updates in here
		
		LevelTimer=LevelTimer+1
		MenuBkgUpdate()		
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	

	
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False		
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=5
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption+1
			If CurrentMenuOption=6 Then CurrentMenuOption=0
			CursorKeyPressed=True
		EndIf
	Else If (KeyDown(28) Or KeyDown(156))
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CursorKeyPressed=True
			If CurrentMenuOption=0 
				If OptionsSoundLevel=1
					OptionsSoundLevel=.6
					
				Else If OptionsSoundLevel=.6
					OptionsSoundLevel=.2
				Else If OptionsSoundLevel=.2
					OptionsSoundLevel=0
				Else
					OptionsSoundLevel=1
				EndIf
			Else If CurrentMenuOption=1 
				If OptionsMusicLevel=1
					OptionsMusicLevel=.7
					
				Else If OptionsMusicLevel=.7
					OptionsMusicLevel=.4
				Else If OptionsMusicLevel=.4
					OptionsMusicLevel=0
				Else
					OptionsMusicLevel=1
				EndIf
				If DoesSoundExist=True Then	ChannelVolume Music,OptionsMusicLevel
			;Else If CurrentMenuOption=2 
			;	Repeat
			;		MyGfxMode=MyGfxMode+1
			;	Until MyGfxModeExists(MyGfxMode)=1 Or MyGfxMode=4
			;	If MyGfxMode=4
			;		MyGfxMode=0
			;	EndIf

			Else If CurrentMenuOption=2 
				If DetailLevel=5
					DetailLevel=3
					
				Else If DetailLevel=3
					DetailLevel=1
				Else 
					DetailLevel=5
				EndIf
				
			;Else If CurrentMenuOption=4
			;	If MyWindowed=0
			;		MyWindowed=1
			;	Else
			;		MyWindowed=0
			;	EndIf
			;Else If CurrentMenuOption=5
			;	Language=Language+1
			;	If Language=3 Then Language=0
			Else If  CurrentMenuOption=3
				NoTimeLimit=1-NoTimeLimit
		;	Else If CurrentMenuOption=4
		;		LockWorlds=1-LockWorlds
		;	Else If CurrentMenuOption=5
		;		OptionsTeleport=1-OptionsTeleport
			Else If CurrentMenuOption=4
				EndMenu()
				; go to site!
				VersionNo1=-996
				StartMenu(15)
				
			Else If CurrentMenuOption=5
				EndMenu()
				SaveOptions()
				StartMenu(1)
			
			EndIf
			
		EndIf
	Else If KeyDown(1)
		If CursorKeyPressed=False
			CursorKeyPressed=True
			EndMenu()
			SaveOptions()
			StartMenu(1)
		EndIf

	Else
		CursorKeyPressed=False
	EndIf


	
End Function

Function MenuLoop10()

	; Display Static Text
	
	If Language=0	
		CenterText("Exit to Windows?",7,1,50)
		CenterText("----------------",8,1,51)
		CenterText("SURE THING",10,1,52)
		CenterText("NO THANKS",11,1,53)
	Else If Language=1
		CenterText("Wonderland Beenden?",7,1,50)
		CenterText("--------------------",8,1,51)
		CenterText("JA BITTE",10,1,52)
		CenterText("NEIN DANKE",11,1,53)
	Else If Language=2
		CenterText("Sortie vers Windows?",7,1,50)
		CenterText("--------------------",8,1,51)
		CenterText("BIEN SUR",10,1,52)
		CenterText("NON MERCI",11,1,53)

	EndIf
	
		PositionTexture menutexture(12),0,watertexv
	RotateEntity menuentity(12),0,0,menutimer/10 Mod 360
	
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(10+CurrentMenuOption)*.044+.027,1.1
	ScaleEntity MenuSelector,11.5*.012,.02,0.01
	

	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next



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

		
		; do the updates in here
		
		LevelTimer=LevelTimer+1
		MenuBkgUpdate()
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	
	
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False		
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=1
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption+1
			If CurrentMenuOption=2 Then CurrentMenuOption=0
			CursorKeyPressed=True
		EndIf
	Else If (KeyDown(28) Or KeyDown(156))
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CursorKeyPressed=True
			If CurrentMenuOption=0 
				
				If Version<10
					EndMenu()
					StartMenu(11)
				Else
					EndMenu()
					GameExit=True
				EndIf
				
			Else
				EndMenu()
				StartMenu(1)
			EndIf
			
		EndIf
	Else If KeyDown(1)
		If CursorKeyPressed=False
			CursorKeyPressed=True
			
			If Version<10
				EndMenu()
				StartMenu(11)
			Else
				EndMenu()
				GameExit=True
			EndIf

			
		EndIf

	Else
		CursorKeyPressed=False
	EndIf


	
End Function

Function MenuLoop11()

	; Display Static Text
	
;	If Language=0	
		CenterText("GET THE FULL VERSION",16,1,52)
		CenterText("NO THANKS",17,1,53)
;	EndIf

	
	CenterText("Get the full version of this game and ...",10,1,54)
	
	If leveltimer Mod 800>700
		Centertext("",13,1,55)
		centertext("",14,1,56)
		centertext("",15,1,57)
	Else If leveltimer Mod 4800<800
		Centertext("Meet two new characters - 'Qookie',",12,1,55)
		centertext("and 'Peegue' - and solve puzzles",13,1,56)
		centertext("using all four players.",14,1,57)
	Else If leveltimer Mod 4800<1600
		Centertext("Explore the world of Wonderland:",12,1,55)
		centertext("travel to Fire Island, venture into",13,1,56)
		centertext("the Creepy Keep, and much, much more.",14,1,57)
	Else If leveltimer Mod 4800<2400
		Centertext("Play over one hundred (!) new and exciting",12,1,55)
		centertext("levels filled with clever puzzles,",13,1,56)
		centertext("new challenges, and hidden surprises.",14,1,57)
	Else If leveltimer Mod 4800<3200
		Centertext("Enjoy many more puzzle elements, including",12,1,55)
		centertext("prisms, box generators, UFOs, Rainbow Spirits,",13,1,56)
		centertext("and much, much more.",14,1,57)
	Else If leveltimer Mod 4800<4000
		Centertext("Unlock the mysteries of the Secret Worlds.",12,1,55)
		centertext("Search for the hidden bonus levels, find the",13,1,56)
		centertext("magical rainbow stars, and save the Stinkers!",14,1,57)

	Else 
		Centertext("Find out why the fun filled adventures",12,1,55)
		centertext("of the Wonderland games have become a",13,1,56)
		centertext("player favourite around the globe.",14,1,57)


	EndIf


	
	PositionTexture menutexture(12),0,watertexv
	RotateEntity menuentity(12),0,0,menutimer/10 Mod 360
	
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(16+CurrentMenuOption)*.044+.027,1.1
	ScaleEntity MenuSelector,24.5*.012,.02,0.01
	

	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next



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

		
		; do the updates in here
		
		LevelTimer=LevelTimer+1
		MenuBkgUpdate()
		
		For i=0 To 5
			PositionEntity boxentity(i),-0.5+1.5*Sin(leveltimer/4+i*60),1.2-0.2*Cos(Leveltimer/4+i*60),4.5-1.5*Cos(Leveltimer/4+i*60)
			TurnEntity boxentity(i),0,0.05*Sin(leveltimer+i*60),0
		Next
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	
	
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False		
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=1
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption+1
			If CurrentMenuOption=2 Then CurrentMenuOption=0
			CursorKeyPressed=True
		EndIf
	Else If (KeyDown(28) Or KeyDown(156))
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CursorKeyPressed=True
			
			For i=0 To 5	
				FreeEntity boxentity(i)
				FreeTexture teleporttexture(i)
			Next
			
			If CurrentMenuOption=0 
			
				
		
				
				VersionNo1=-998
				EndMenu()
				GameExit=True
								
			Else
				EndMenu()
				GameExit=True

			EndIf
			
		EndIf
	Else If KeyDown(1)
		If CursorKeyPressed=False
			CursorKeyPressed=True
			
			For i=0 To 5	
				FreeEntity boxentity(i)
				FreeTexture teleporttexture(i)
			Next
			
			EndMenu()
			GameExit=True


			
		EndIf

	Else
		CursorKeyPressed=False
	EndIf





	
End Function



Function MenuLoop15()

	; Display Static Text
	
	If Language=0	
		CenterText("Go Online?",4,1,50)
		CenterText("----------",5,1,51)
		CenterText("This option will exit Secret Worlds",7,1,52)
		CenterText("and connect you to an Internet Website.",8,1,53)
		
		CenterText("PROCEED",10,1,54)
		CenterText("NO THANKS",11,1,55)
	Else If Language=1
		CenterText("Wonderland Beenden?",7,1,50)
		CenterText("--------------------",8,1,51)
		CenterText("JA BITTE",10,1,52)
		CenterText("NEIN DANKE",11,1,53)
	Else If Language=2
		CenterText("Sortie vers Windows?",7,1,50)
		CenterText("--------------------",8,1,51)
		CenterText("BIEN SUR",10,1,52)
		CenterText("NON MERCI",11,1,53)

	EndIf
	
	
	
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(10+CurrentMenuOption)*.044+.027,1.1
	ScaleEntity MenuSelector,11.5*.012,.02,0.01
	
		PositionTexture menutexture(12),0,watertexv
	RotateEntity menuentity(12),0,0,menutimer/10 Mod 360
	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next



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

		
		; do the updates in here
		
		LevelTimer=LevelTimer+1
		MenuBkgUpdate()
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	
	
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False		
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=1
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption+1
			If CurrentMenuOption=2 Then CurrentMenuOption=0
			CursorKeyPressed=True
		EndIf
	Else If (KeyDown(28) Or KeyDown(156))
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CursorKeyPressed=True
			If CurrentMenuOption=0 
				
				EndMenu()
				GameExit=True
				
			Else
				EndMenu()
				VersionNo1=oldversionno1
				StartMenu(1)
			EndIf
			
		EndIf
	Else If KeyDown(1)
		If CursorKeyPressed=False
			CursorKeyPressed=True
			EndMenu()
			VersionNo1=oldversionno1
			StartMenu(1)
		EndIf

	Else
		CursorKeyPressed=False
	EndIf


	
End Function

Function MenuLoop18()

	; Display Static Text
	
	
	nofpics=1
	If Language=0	
		CenterText("VIEW UNLOCKED STORY PICTURES",4,1,50)
		CenterText("----------------------------",5,1,51)
		
		CenterText("Secret Worlds Introduction",7,1,52)
		
		For i=8 To 0 Step -1
			If storymenupic(i)=1
				nofpics=i+2
				Exit
			EndIf
		Next
				
		If nofpics>1
			CenterText("Meet Loof",8,1,53)
		EndIf
		If nofpics>2
			CenterText("Entering Wondertown",9,1,54)
		EndIf
		If nofpics>3
			CenterText("Meet Peegue",10,1,55)
		EndIf
		If nofpics>4
			CenterText("The Creepy Keep",11,1,56)
		EndIf
		If nofpics>5
			CenterText("Meet Qookie",12,1,57)
		EndIf
		If nofpics>6
			CenterText("Fire Island",13,1,58)
		EndIf
		If nofpics>7
			CenterText("Forever Forest",14,1,59)
		EndIf
		If nofpics>8
			CenterText("The Void",15,1,60)
		EndIf
		If nofpics>9
			CenterText("Happy Ending",16,1,61)
		EndIf

		centertext("MAIN MENU",7+nofpics,1,62)
		If nofpics<10 centertext("(complete levels to unlock further pictures)",18,1,63)
	EndIf
	
	
	
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(7+CurrentMenuOption)*.044+.027,1.1
	ScaleEntity MenuSelector,26.5*.012,.02,0.01
	
		PositionTexture menutexture(12),0,watertexv
	RotateEntity menuentity(12),0,0,menutimer/10 Mod 360
	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next



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

		
		; do the updates in here
		
		LevelTimer=LevelTimer+1
		MenuBkgUpdate()
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	
	
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False		
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=nofpics
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption+1
			If CurrentMenuOption=nofpics+1 Then CurrentMenuOption=0
			CursorKeyPressed=True
		EndIf
	Else If (KeyDown(28) Or KeyDown(156))
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CursorKeyPressed=True
			If CurrentMenuOption=nofpics 
				endmenu()
				startmenu(1)
			Else
				If currentmenuoption=0
					endmenu()
					storymenuselected=1
					startstoryscreen(1)
				Else If currentmenuoption=1
					endmenu()
					storymenuselected=1
					startstoryscreen(6)
				Else If currentmenuoption=2
					endmenu()
					storymenuselected=1
					startstoryscreen(9)
				Else If currentmenuoption=3
					endmenu()
					storymenuselected=1
					startstoryscreen(10)
				Else If currentmenuoption=4
					endmenu()
					storymenuselected=1
					startstoryscreen(21)
				Else If currentmenuoption=5
					endmenu()
					storymenuselected=1
					startstoryscreen(24)
				Else If currentmenuoption=6
					endmenu()
					storymenuselected=1
					startstoryscreen(25)
				Else If currentmenuoption=7
					endmenu()
					storymenuselected=1
					startstoryscreen(30)
				Else If currentmenuoption=8
					endmenu()
					storymenuselected=1
					startstoryscreen(35)
				Else If currentmenuoption=9
					endmenu()
					storymenuselected=1
					startstoryscreen(45)
			
		
				EndIf
			EndIf
			
		EndIf
	Else If KeyDown(1)
		If CursorKeyPressed=False
			CursorKeyPressed=True
			EndMenu()
			StartMenu(1)
		EndIf

	Else
		CursorKeyPressed=False
	EndIf


	
End Function
Function MenuLoop19()

	; Display Static Text
	i#=21.0-((Float(leveltimer)/200.0) Mod 76)
	
		CenterText("WONDERLAND SECRET WORLDS",-2+i,1,51)
		CenterText("A MIDNIGHT SYNERGY PRODUCTION",0+i,1,50)
		
		CenterText("DESIGN, CODE, AUDIOVISUALS",2+i,1,52)
		CenterText("Patrick Maidorn",3+i,1,53)
		
		CenterText("LEVEL DESIGNERS",5+i,1,54)
		CenterText("Malcolm Fowler",6+i,1,55)
		CenterText("Janet Galle",7+i,1,56)
		CenterText("Dean J. Lemon",8+i,1,57)
		CenterText("Carolyn Kinney",9+i,1,58)
		CenterText("Julie Majewski",10+i,1,59)
		CenterText("Roger Melin",11+i,1,60)
		CenterText("Pauline Robinson",12+i,1,61)
		CenterText("Carolyn Schindler",13+i,1,62)
		CenterText("Steve Seaman",14+i,1,63)
		CenterText("Mette Sidelmann",15+i,1,64)


		centertext("STORY ILLUSTRATIONS",17+i,1,65)
		centertext("James Abraham",18+i,1,66)
		
		centertext("MUSIC",20+i,1,67)
		centertext("Jonne Valtonen",21+i,1,68)
		
		CenterText("ADDITIONAL 3D GRAPHICS",23+i,1,69)
		centertext("Danny Hawrysio",24+i,1,70)
		centertext("Dean J. Lemon",25+i,1,71)
		centertext("Pauline Robinson",26+i,1,72)
		centertext("Steve Seaman",27+i,1,73)
		
		centertext("SOFTWARE DEVELOPMENT CONSULTANT",29+i,1,88)
		centertext("Claire Polster",30+i,1,89)
		
		centertext("TESTERS",32+i,1,74)
		centertext("Bearsland",33+i,1,75)
		centertext("carolyn",34+i,1,76)
		centertext("Maureen Davidson",35+i,1,77)
		centertext("Gen",36+i,1,78)
		centertext("jperleevt",37+i,1,79)
		centertext("Kaboom!",38+i,1,80)
		centertext("Lori",40+i,1,81)
		centertext("Lillian",39+i,1,82)
		centertext("Pat",41+i,1,83)
		centertext("Ramonda",42+i,1,84)
		centertext("Robbie",43+i,1,85)



		
		centertext("THANK YOU TO THE USUAL SUSPECTS",45+i,1,87)
		

		
		
		
	
	
	
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(20+CurrentMenuOption)*.044+.027,1.1
	
	
		PositionTexture menutexture(12),0,watertexv
	RotateEntity menuentity(12),0,0,menutimer/10 Mod 360
	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next



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

		
		; do the updates in here
		
		LevelTimer=LevelTimer+1
		MenuBkgUpdate()
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	
	
	If (KeyDown(28) Or KeyDown(156) Or KeyDown(1))
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CursorKeyPressed=True
			
			EndMenu()
			
			StartMenu(1)
			
			
		EndIf
	
	Else
		CursorKeyPressed=False
	EndIf


	
End Function





Function EndMenu()

	
	
	HideEntity MenuSelector
	For i=0 To 79
		If MenuEntity(i)<>0 HideEntity MenuEntity(i)
	Next
	
	; Fonts
	For l.LetterType=Each LetterType
		FreeEntity l\Entity
		Delete l
	Next



End Function
Function StartLevelMusic()

	If doessoundexist=False Then Return

	If version<10 And currentlevel=8 Then Return

	If Currentlevel<8
		If currentmusic<>3
			StopChannel Music
			Music=PlayMusic("res5.dat")	; garden
			currentmusic=3
		EndIf
	Else If currentlevel<24
		If currentmusic<>2
			StopChannel Music
			music=PlayMusic("res3.dat") ; puzzle
			currentmusic=2
		EndIf
	Else If currentlevel<35 And currentlevel<>32
		If currentmusic<>3
			StopChannel Music
			music=PlayMusic("res5.dat") ; garden
			currentmusic=3
		EndIf
	Else If	currentlevel=32
		If currentmusic<>4
			StopChannel Music
			music=PlayMusic("res6.dat") ; dungeon
			currentmusic=4
		EndIf

	Else If currentlevel<47
		If currentmusic<>6
			StopChannel Music
			music=PlayMusic("res8.dat") ; spooky
			currentmusic=6
		EndIf
	Else If currentlevel<59 And currentlevel<>53
		If currentmusic<>2
			StopChannel Music
			music=PlayMusic("res3.dat") ; puzzle
			currentmusic=2
		EndIf
	

	Else If currentlevel<63 Or currentlevel=53
		If currentmusic<>4
			StopChannel Music
			music=PlayMusic("res6.dat") ; dungeon
			currentmusic=4
		EndIf

	Else If currentlevel<74
		If currentmusic<>5
			StopChannel Music
			music=PlayMusic("res7.dat") ; slow
			currentmusic=5
		EndIf
	Else If currentlevel<84
		If currentmusic<>4
			StopChannel Music
			music=PlayMusic("res6.dat") ; dungeon
			currentmusic=4
		EndIf
	Else If currentlevel<97
		If currentmusic<>3
			StopChannel Music
			music=PlayMusic("res5.dat") ; garden
			currentmusic=3
		EndIf
	Else If currentlevel<101
		If currentmusic<>2
			StopChannel Music
			music=PlayMusic("res3.dat") ; puzzle
			currentmusic=2
		EndIf
	Else If currentlevel<102
		If currentmusic<>6
			StopChannel Music
			music=PlayMusic("res8.dat") ; spooky
			currentmusic=6
		EndIf
		
	Else If currentlevel<107
		If currentmusic<>2
			StopChannel Music
			music=PlayMusic("res3.dat") ; puzzle
			currentmusic=2
		EndIf
	Else If currentlevel<116
		If currentmusic<>6
			StopChannel Music
			music=PlayMusic("res8.dat") ; spooky
			currentmusic=6
		EndIf
	Else If currentlevel<119
		If currentmusic<>2
			StopChannel Music
			music=PlayMusic("res3.dat") ; puzzle
			currentmusic=2
		EndIf



		
	Else If currentlevel<126
		If currentmusic<>1
			StopChannel Music
			music=PlayMusic("res4.dat") ; intro
			currentmusic=1
		EndIf
	Else If Currentlevel<127
		If currentmusic<>3
			StopChannel Music
			Music=PlayMusic("res5.dat")	; garden
			currentmusic=3
		EndIf
	Else If Currentlevel<129
		If currentmusic<>4
			StopChannel Music
			Music=PlayMusic("res6.dat")	; dungeon
			currentmusic=4
		EndIf
	Else If Currentlevel<132
		If currentmusic<>3
			StopChannel Music
			Music=PlayMusic("res5.dat")	; garden
			currentmusic=3
		EndIf




	EndIf
	
	; res3 - music 2
	; res5 - music 3
	; res6 - music 4
	; 
	
	If Currentmusic=1 Or currentmusic=5
		ChannelVolume music,OptionsMusicLevel/2.0
	Else
		ChannelVolume music,OptionsMusicLevel
	EndIf


End Function
Function CreateNewPlayer()
	; reset all data
	CurrentWorld=0
	For i=0 To PlayGroundLevel-1
		CurrentPlayerLevelScore(i)=0
	Next
	
	CurrentPlayerID=0
	For i=1 To Len(CurrentPlayername$)
		CurrentPlayerID=CurrentPlayerID+Asc(Mid$(currentplayername$,i,1))
	Next
	CurrentPlayerID=CurrentPlayerID+MilliSecs()
		
	; and create new player file on disk
	If CurrentPlayerName$="" Then CurrentPlayerName$="ANONYMOUS"
	
	File=ReadFile("players/"+Lower$(CurrentPlayerName$)+".pla")

	If File=0
		;CloseFile(File)
	Else
		CloseFile(File)
		CopyFile "players/"+Lower$(CurrentPlayerName$)+".pla","players/"+Lower$(CurrentPlayerName$)+".bak"
	EndIf

	
	File=WriteFile("players/"+Lower$(CurrentPlayerName$)+".pla")
	
	;*********************** LATER:

	; Need error handling if file dne
	
	;*****************************

	j=0
	WriteString(File,"Stinky & Loof WSW Player File")	
	WriteString(File,CurrentPlayerName$)
	WriteInt (File,CurrentPlayerID)
	CurrentLevel=1
	WriteInt (File,0)
	WriteInt (File,j)		; # of levels completed
	WriteInt (File,j)		; total score
	
	
	getworldmapinfo()
	

	
	For i=0 To 140
		WorldMapLevelScore(i)=0
		WriteInt File,WorldMapLevelStatus(i)
		WriteInt File,WorldMapLevelScore(i)
		WriteInt File,WorldMapLevelType(i)
	Next
	
	PlayerGoldStars=0
	WriteInt File,PlayerGoldStars
	PlayerRainbowStars=0
	WriteInt File,PlayerRainbowStars
	PlayerPeeguesPass=0
	WriteInt File,PlayerPeeguesPass
	PlayerQookiesMap=0
	WriteInt File,PlayerQookiesMap
	PlayerForestMap=0
	WriteInt File,PlayerForestMap

	

	For i=0 To 19
		storytold(i)=0
		WriteInt file,storytold(i)
	Next
	

	CloseFile(File)
	
	
	
	

End Function
Function LoadPlayer(FileName$)
	File=ReadFile("players/"+Lower$(FileName$)+".pla")
	
	;*********************** LATER:

	; Need error handling if file dne
	
	;*****************************

	CurrentPlayerName$=ReadString$(File)	; just the "player file" message
	CurrentPlayerName$=ReadString$(File)
	CurrentPlayerID=ReadInt(File)
	CurrentLevel= ReadInt(File)
	j=ReadInt(File)		; # of levels completed - don't need these at this stage
	j=ReadInt(File)		; total score
	For i=0 To 140
		WorldMapLevelStatus(i)=ReadInt(File)
		WorldMapLevelScore(i)=ReadInt(File)
		WorldMapLevelType(i)=ReadInt(File)

	Next
	
	PlayerGoldStars=ReadInt(File)

	PlayerRainbowStars=ReadInt(File)

	PlayerPeeguesPass=ReadInt(File)

	PlayerQookiesMap=ReadInt(File)

	PlayerForestMap=ReadInt(File)


	For i=0 To 19
		storytold(i)=ReadInt(file)
	Next

	
	CloseFile(File)
	
	

		
	
	
End Function

Function SavePlayer()
	; saves teh current player
	
	File=WriteFile("players/"+Lower$(CurrentPlayerName$)+".pla")
	
	If File=0
		Return
	EndIf

	
	
	j=0
	j2=0
	WriteString(File,"Stinky & Loof WSW Player File")	
	WriteString(File,CurrentPlayerName$)
	WriteInt (File,CurrentPlayerID)
	WriteInt (File,CurrentLevel)
	
	

	
	For i=1 To 140
		If WorldMapLevelScore(i)>0
			j=j+1
			j2=j2+WorldMapLevelScore(i)
		EndIf
	Next
					
	WriteInt (File,j)		; # of levels completed
	WriteInt (File,j2)		; total score
	
	
	For i=0 To 140
		WriteInt File,WorldMapLevelStatus(i)
		WriteInt File,WorldMapLevelScore(i)
		WriteInt File,WorldMapLevelType(i)
	Next
	
	
	WriteInt File,PlayerGoldStars
	
	WriteInt File,PlayerRainbowStars
	
	WriteInt File,PlayerPeeguesPass
	
	WriteInt File,PlayerQookiesMap
	
	WriteInt File,PlayerForestMap
	
	For i=0 To 19
		WriteInt file,storytold(i)
	Next




	CloseFile(File)
	
	
	
End Function

Function MenuBkgUpdate()

	MenuTimer=MenuTimer+1
	Watertexu#=Watertexu#+0.001
	Watertexv#=Watertexv#+0.0005
	
	
	
	

End Function

Function EndBkgUpdate()

	;If LevelTimer>=1440 Leveltimer=0
	
	TurnEntity MenuEntity(60),0,0.1,0
	PositionTexture Rainbowtexture,Float(LevelTimer)/1000.0,Float(LevelTimer)/2000.0
	
	; star movements (in steps of Leveltimer Mod 720)
	Select EndBkgStyle
	Case 0
		
		For i=0 To 49
			PositionEntity MenuEntity(i+10),(3+Sin(Leveltimer))*Cos(i*36),(3+Sin(Leveltimer))*Sin(i*36),i*5-((LevelTimer-i*5) Mod 250)
			RotateEntity MenuEntity(i+10),0,0,leveltimer*3+i*47

		Next
			
	
	
	
	
	End Select
	
	t2=leveltimer Mod 7800
	
	If t2=5;stinky
		If MenuEntity(61)<>0 FreeEntity MenuEntity(61)
		MenuEntity(61)=CopyEntity(PlayerMesh(1))
	 	EntityTexture MenuEntity(61),PlayerTexture(1)
		PositionEntity MenuEntity(61),0,0,100
		TurnEntity MenuEntity(61),20,180,0
		AnimateMD2 MenuEntity(61),1,0.1,44,52
		
	EndIf
	If t2=605;loof
		If MenuEntity(61)<>0 FreeEntity MenuEntity(61)
		MenuEntity(61)=CopyEntity(PlayerMesh(2))
	 	EntityTexture MenuEntity(61),PlayerTexture(2)
		;PositionMesh MenuEntity(61),0,-0.5,0
		PositionEntity MenuEntity(61),0,0,100
		TurnEntity MenuEntity(61),0,180,0
		AnimateMD2 MenuEntity(61),2,.1,38,44
		
	EndIf
	
	If t2=1205;scouge 
		If MenuEntity(61)<>0 FreeEntity MenuEntity(61)
		MenuEntity(61)=CopyEntity(CannonMesh)
	 	;EntityTexture MenuEntity(61),LoofTexture
		PositionEntity MenuEntity(61),0,0,100
		TurnEntity MenuEntity(61),0,0,180
		;AnimateMD2 MenuEntity(61),1,0.5,40,43
		
	EndIf

	
	If t2=1805;coily
		If MenuEntity(61)<>0 FreeEntity MenuEntity(61)
		MenuEntity(61)=CopyEntity(MonsterMesh(0))
	 	;EntityTexture MenuEntity(61),LoofTexture
		PositionEntity MenuEntity(61),0,0,100
		;TurnEntity MenuEntity(61),-20,180,0
		;AnimateMD2 MenuEntity(61),1,0.5,40,43
		
	EndIf
	
	If t2=2405;kaboom
		If MenuEntity(61)<>0 FreeEntity MenuEntity(61)
		MenuEntity(61)=CopyEntity(MonsterMesh(3))
	 	;EntityTexture MenuEntity(61),LoofTexture
		PositionEntity MenuEntity(61),0,0,100
		;TurnEntity MenuEntity(61),-20,180,0
		;AnimateMD2 MenuEntity(61),1,0.5,40,43
		
	EndIf
	
	If t2=3005;z-bot
		If MenuEntity(61)<>0 FreeEntity MenuEntity(61)
		MenuEntity(61)=CopyEntity(MonsterMesh(1))
	 	;EntityTexture MenuEntity(61),LoofTexture
		PositionEntity MenuEntity(61),0,0,100
		TurnEntity MenuEntity(61),-90,0,0
		;AnimateMD2 MenuEntity(61),1,0.5,40,43
		
	EndIf
	
	If t2=3605;saucer
		If MenuEntity(61)<>0 FreeEntity MenuEntity(61)
		MenuEntity(61)=CopyEntity(MonsterMesh(5))
	 	;EntityTexture MenuEntity(61),LoofTexture
		PositionEntity MenuEntity(61),0,0,100
		TurnEntity MenuEntity(61),90,0,0
		;AnimateMD2 MenuEntity(61),1,0.5,40,43
		
	EndIf
	
	If t2=4205;
		If MenuEntity(61)<>0 FreeEntity MenuEntity(61)
		MenuEntity(61)=CopyEntity(MonsterMesh(7))
	 	;EntityTexture MenuEntity(61),LoofTexture
		PositionEntity MenuEntity(61),0,0,100
		TurnEntity MenuEntity(61),10,180,0
		AnimateMD2 MenuEntity(61),2,.3,0,6

		
	EndIf
	
	If t2=4805
		If MenuEntity(61)<>0 FreeEntity MenuEntity(61)
		MenuEntity(61)=CopyEntity(MonsterMesh(7))
	 	EntityTexture MenuEntity(61),MonsterTexture(8)
		PositionEntity MenuEntity(61),0,0,100
		TurnEntity MenuEntity(61),10,180,0
		AnimateMD2 MenuEntity(61),2,.3,0,6

		
	EndIf
	
	If t2=5405
		If MenuEntity(61)<>0 FreeEntity MenuEntity(61)
		MenuEntity(61)=CopyEntity(MonsterMesh(9))
	 	;EntityTexture MenuEntity(61),LoofTexture
		PositionEntity MenuEntity(61),0,0,100
		;TurnEntity MenuEntity(61),90,0,0
		;AnimateMD2 MenuEntity(61),1,0.5,40,43
		
	EndIf








	
	
	t=leveltimer Mod 600
	
	If t2>610 And t2<1200
		TurnEntity menuentity(61),0,0,-2
	EndIf
	If t2>1810 And t2<2400
		TurnEntity menuentity(61),1,2,-2
	EndIf
	If t2>3010 And t2<3600
		TurnEntity menuentity(61),0,0,-3
	EndIf
	If t2>3610 And t2<4200
		TurnEntity menuentity(61),0,-3,0
	EndIf
	If t2>5410 And t2<6000
		TurnEntity menuentity(61),2,-3,1
	EndIf




	
	If t>=10	And t2<6000	
		PositionEntity MenuEntity(61),0,-.6,98.0-Float(t)/6.0
		If t=10
			EntityAlpha MenuEntity(61),1
		EndIf
		If t>550
			EntityAlpha MenuEntity(61),(600.0-Float(t))/50.0
		EndIf
	EndIf

		
	

End Function

Function EndBkgUpdateDeluxe()

	;If LevelTimer>=1440 Leveltimer=0
	
	TurnEntity MenuEntity(60),0,0.1,0
	PositionTexture Rainbowtexture,Float(LevelTimer)/1000.0,Float(LevelTimer)/2000.0
	
	; star movements (in steps of Leveltimer Mod 720)
	Select EndBkgStyle
	Case 0
		
		For i=0 To 49
			If i>0 
				ScaleEntity MenuEntity(i+10),.4,.4,0.01
			Else 
				ScaleEntity MenuEntity(i+10),1,1,0.01
			EndIf
			PositionEntity MenuEntity(i+10),24*Cos(Float(LevelTimer-i*16.0)/16.0),(8*Sin(Float(LevelTimer-i*16.0)/16.0)+Float(i)/40.0*Sin(i*57+i*i*15)),25+Float(i)/100.0;  (20*Sin(leveltimer+Float(i)*3.6))*Cos((Float(Leveltimer)+Float(i)*4.0*7.2)/4.0),0.4,(20+12*Sin(leveltimer+Float(i)*3.6))*Sin((Float(Leveltimer)+Float(i)*7.2*4.0)/4.0)
			RotateEntity MenuEntity(i+10),0,0,leveltimer+i*47

		Next
		
			
	
	
	
	
	End Select
	
	If LevelTimer=0
		; create 4 chars, planet, sky
		For i=61 To 66
			If MenuEntity(i)<>0 FreeEntity MenuEntity(i)
			MenuEntity(i)=0
		Next
		For i=1 To 4
			MenuEntity(60+i)=CopyEntity(PlayerMesh(i))
		 	EntityTexture MenuEntity(60+i),PlayerTexture(i)
			PositionEntity MenuEntity(60+i),-3.75+i*1.5,0,0
			TurnEntity MenuEntity(60+i),20,180+25-i*10,0
			AnimateMD2 MenuEntity(60+i),1,Rnd(0.04,0.10),44,52
		Next
		MenuEntity(65)=CreatePlane()
		BackGroundTexture=MyLoadTexture ("graphics\greenfloor1.jpg",1,False)
		EntityTexture MenuEntity(65),BackGroundTexture
		ScaleTexture Backgroundtexture,3,3
		TranslateEntity MenuEntity(65),0,0,0
		
		MenuEntity(66)=CreatePlane()
		BackGroundTexture2=MyLoadTexture ("graphics\stars.jpg",1,False)
		
		ScaleTexture BackGroundTexture2,10,10
		
		EntityTexture MenuEntity(66),BackGroundTexture2
		RotateEntity MenuEntity(66),-90,0,0
		TranslateEntity MenuEntity(66),0,0,30
		
		

		
	EndIf
	
	PositionTexture BackGroundTexture2,Float(LevelTimer Mod 10000)/10000.0,Float(LevelTimer Mod 20000)/20000.0
		
	
	;TranslateEntity Camera,0,.001,-.01
	



	

End Function

Function LoadOptions()
	File=ReadFile("options.cfg")
	
	If File=0
		OptionsSoundLevel=1
		OptionsMusicLevel=1
		DetailLevel=3
		MyGfxMode=1
		MyColMode=1
		MyWindowed=0
		Language=0
		NoTimeLimit=0
		LockWorlds=0
		OptionsTeleport=1
		Return
	EndIf
	
	;*********************** LATER:

	; Need error handling if file dne
	
	;*****************************

	;ex$=ReadString$(File)
	OptionsSoundLevel2=ReadInt(File)
	OptionsSoundLevel=Float(OptionsSoundLevel2)/100.0
	OptionsMusicLevel2=ReadInt(File)
	OptionsMusicLevel=Float(OptionsMusicLevel2)/100.0
	DetailLevel= ReadInt(File)
	MyGfxMode=ReadInt(File)
	MyColMode=ReadInt(File)
	MyWindowed=ReadInt(File)
	Language=ReadInt(File)
	NoTimeLimit=ReadInt(File)
	LockWorlds=ReadInt(File)
	MyColModeEd=ReadInt(File)
	MyWindowedEd=ReadInt(File)
	OptionsTeleport=ReadInt(File)
	

	CloseFile(File)
	
	
	
	
	
		
	
End Function


Function SaveOptions()
	; saves teh current options
	
	File=WriteFile("options.cfg")
	
	If File=0
		Return
	EndIf
	
	
	;*********************** LATER:

	; Need error handling if file dne
	
	;*****************************
	
	j=0
	j2=0
	
	
	
	;WriteString(File,"Wonderland Options File")	
	WriteInt File,Int(OptionsSoundLevel*100)
	WriteInt File,Int(OptionsMusicLevel*100)
	WriteInt File,DetailLevel
	WriteInt File,MyGfxMode
	WriteInt File,MyColMode
	WriteInt File,MyWindowed
	WriteInt File,Language
	WriteInt File,NoTimeLimit
	WriteInt File,LockWorlds
	WriteInt File,MyColModeEd
	WriteInt File,MyWindowedEd
	WriteInt File,OptionsTeleport
	

	
	CloseFile(File)
	
	
	
	
	
End Function

Function SaveGameState()
	
	file=WriteFile("savestate.wld")
	If file=0 Return
	
	WriteInt File,MapWidth
	WriteInt File,MapHeight
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			WriteInt File,MapTile(i,j)
		Next
	Next
	WriteInt File,Leveltimer
	WriteInt File,CurrentLevelScore
	WriteInt File,LevelCompleted
	WriteInt File,LevelCompletedTimer
	WriteInt File,BothCompleted
	WriteInt File,MapTime
	WriteInt File,MapMilliTime
	WriteInt File,FocusSwitchTimer
	;WriteInt File,
	;WriteInt File,
	;WriteInt File,
	
	;camera
	WriteFloat File,CameraDX#
	WriteFloat File,CameraDY#	
	WriteFloat File,CameraDZ#
	WriteFloat File,CameraDStep#
	WriteFloat File,FreeCameraX#
	WriteFloat File,FreeCameraZ#
	WriteFloat File,EntityX(Camera,True)
	WriteFloat File,EntityY(Camera,True)
	WriteFloat File,EntityZ(Camera,True)

	
	
	;player
	For k=1 To 4
		WriteInt File,PlayerExists(k)
		WriteInt File,FocusOnPlayer(k)
	Next

	
	
	CloseFile file
	SaveGameStateActive=True

End Function
Function LoadGameState()

	file=ReadFile("savestate.wld")
	If file=0 Return
	

	MapWidth=ReadInt(File)
	MapHeight=ReadInt(File)
	For j=0 To MapHeight-1
		For i=0 To MapWidth-1
			MapTile(i,j)=ReadInt(File)
		Next
	Next
	Leveltimer=ReadInt(File)
	CurrentLevelScore=ReadInt(File)
	LevelCompleted=ReadInt(File)
	LevelCompletedTimer=ReadInt(File)
	BothCompleted=ReadInt(File)
	MapTime=ReadInt(File)
	MapMilliTime=ReadInt(File)
	FocusSwitchTimer=ReadInt(File)
	;=ReadInt(File)
	;=ReadInt(File)
	;=ReadInt(File)
	
	;camera
	CameraDX#=ReadFloat(File)
	CameraDY#=ReadFloat(File)	
	CameraDZ#=ReadFloat(File)
	CameraDStep#=ReadFloat(File)
	FreeCameraX#=ReadFloat(File)
	FreeCameraZ#=ReadFloat(File)
	PositionEntity Camera,ReadFloat(File),ReadFloat(File),ReadFloat(File);,True
	
	
	;player 
	For k=1 To 4 
		PlayerExists(k)=ReadInt(File)
		FocusOnPlayer(k)=ReadInt(File)
	Next
	
	



	
	CloseFile file


End Function



Function StartMapScreen()

		

		EndMenu() ; get rid of standard menu background
		CurrentMenu=4
		
		
		; music
		StartLevelMusic()
		
		; load the world map textures and create the map
		
		ShowEntity WorldMap
		ShowEntity WorldMapshade1
		ShowEntity WorldMapshade2
		PositionEntity worldmapshade1,-.5+24.4*.02+.01,0.372-Float(-0.5)*.044+.027,1.2
		PositionEntity worldmapshade2,-.5+24.4*.02+.01,0.372-Float(18)*.044+.027,1.2
		
		
		; Create the WorldMapLevels - 
		; - this is a single surface with all the level dots (0-red,1-green,2-gold,3-empty/secret)
		WorldMapLevels=CreateMesh()
		Surface=CreateSurface(WorldMapLevels)
		size#=.05
		For i=1 To NofWorldMapLevels-1

			u#=0.5*(WorldMapLevelStatus(i) Mod 2)+.25
			v#=0.5*Floor(WorldMapLevelStatus(i) / 2)+.25

			AddVertex(Surface,WorldMapLevelX(i)-size,0.01,-WorldMapLevelY(i)-size,u-.25,v+.25)
			AddVertex(Surface,WorldMapLevelX(i)+size,0.01,-WorldMapLevelY(i)-size,u+.25,v+.25)
			AddVertex(Surface,WorldMapLevelX(i)+size,0.01,-WorldMapLevelY(i)+size,u+.25,v-.25)
			AddVertex(Surface,WorldMapLevelX(i)-size,0.01,-WorldMapLevelY(i)+size,u-.25,v-.25)
			AddTriangle(Surface,2+i*4-4,1+i*4-4,0+i*4-4)
			AddTriangle(Surface,3+i*4-4,2+i*4-4,0+i*4-4)
		
		Next 
		UpdateNormals(WorldMapLevels)
		EntityTexture WorldMapLevels,WorldLevelTexture		
						
		; create the cursor
		WorldMapCursor=CreateFlatSquare(WorldLevelTexture2)
		PositionMesh WorldMapCursor,-.5,0,.5
		ScaleEntity WorldMapCursor,.12,.12,.12
		If worldmapwalking=0
			PositionEntity WorldMapCursor,WorldmapLevelX(CurrentLevel),0.02,-WorldMaplevelY(CurrentLevel)
		Else
			PositionEntity WorldMapCursor,WorldmapLevelX(CurrentLevelbackup),0.02,-WorldMaplevelY(CurrentLevelbackup)
			EndIf							
		RotateEntity Camera,90,0,0
		
	
		If worldmapwalking=0
			PositionEntity Camera,WorldmapLevelX(CurrentLevel),3,-WorldmapLevelY(CurrentLevel)	
		Else
			PositionEntity Camera,WorldmapLevelX(CurrentLevelbackup),3,-WorldmapLevelY(CurrentLevelbackup)
		EndIf
		CameraClsColor Camera,10,82,239	
		
		FreeCameraZ=0
		FreeCameraX=0
						
		UpdateWorld
		
		MenuBrightnessGoal=254
		LightColor Levellight,255,255,255
		AmbientLight 255,255,255
		
		
		
	

End Function



Function mapscreenloop()

	
	;score
	j=0
	j2=0
	For i=1 To 140
		If WorldMapLevelScore(i)>0
			j=j+1
			j2=j2+WorldMapLevelScore(i)
		EndIf
	Next
	ex$="Score: "+Str$(j2)
	DisplayText(ex$,50-Len(ex$),0,1,56)
	ex$="Levels: "+Str$(j)
	DisplayText(ex$,50-Len(ex$),1,1,57)


	If currentlevel=1
		If leveltimer Mod 600 <400
			CenterText("Use Arrow Keys to Move around Map.",18,1,58)
		Else 
			centertext("",1,1,58)
		EndIf
	Else ;If currentlevel<7
		If leveltimer Mod 600 <400
			CenterText("Press 'Enter' to Play/Replay this Level.",18,1,58)
		Else 
			centertext("",171,1,58)
		EndIf
	EndIf
	
	; levelname
	; and get the level name
	
	
	
	CenterText(CurrentWorldLevelName$(currentlevel,0),16,1,59)
	CenterText(CurrentWorldLevelName$(currentlevel,1),17,1,60)

	

		

	
	;inventory
	y=2
	;DisplayText("Inventory",0,0,1,50)
	DisplayText("Gold Stars: "+Str$(PlayerGoldStars),0,0,1,51)
	If PlayerRainbowStars>0
		DisplayText("Rainbow Stars: "+Str$(PlayerRainbowStars),0,1,1,52)
		y=y+1
	EndIf
	If PlayerPeeguesPass>0
		DisplayText(" Peegue's Pass",15,0,1,53)
		y=y+1
	EndIf
	If PlayerQookiesMap>0
		DisplayText(" Qookie's Map",15,0,1,53)
		y=y+1
	EndIf
	If PlayerForestMap>0
		DisplayText(" Forest Map",15,0,1,53)
		y=y+1
	EndIf



	
	
	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next



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

		
		; do the updates in here
		
		LevelTimer=LevelTimer+1
			
		If currentmenu<>7 Then mapscreenupdate()
		;TurnEntity camera,3,2,1
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	
	
	Flip

	
	; Input Etc
	If worldmapwalking=0
	
		If KeyDown(42) Or KeyDown(54)
			; shift key is pressed - Camera control
			ShiftKeyPressed=True
					
			If KeyDown(200) Or KeyDown(72) 
				;If EntityZ(Camera)<-1-9.5
					TranslateEntity Camera,0,0,.02	;forward
					FreeCameraZ=FreeCameraZ+.02
				;EndIf
			EndIf
			If KeyDown(208) Or KeyDown(80) 
				;If EntityZ(Camera)>-(MapHeight-1)-9.5
					TranslateEntity Camera,0,0,-.02	;backward
					FreeCameraZ=FreeCameraZ-.02
				;EndIf
			EndIf
			If KeyDown(75) Or KeyDown(203)
				;If EntityX(Camera)>1
					TranslateEntity Camera,-.02,0,0	;left
					FreeCameraX=FreeCameraX-.02
				;EndIf
			EndIf
			If KeyDown(77) Or KeyDown(205)
				;If EntityX(Camera)<MapWidth-1
					TranslateEntity Camera,.02,0,0	;right
					FreeCameraX=FreeCameraX+.02
				;EndIf
			EndIf
				
		Else
					
			ShiftKeyPressed=False
			If FreeCameraX<>0 Or FreeCameraZ<>0
				TranslateEntity Camera,-FreeCameraX,0,-FreeCameraZ
				FreeCameraX=0
				FreeCameraZ=0
			EndIf

	
	
			If (KeyDown(200) Or KeyDown(72)) ;up
				If CursorKeyPressed=False
					If WorldMapLevelExitNorth(CurrentLevel)>=0
						
						WorldMapWalking=100*Sqr((WorldMapLevelX(WorldMapLevelExitNorth(CurrentLevel))-WorldMapLevelX(CurrentLevel))^2+(WorldMapLevelY(WorldMapLevelExitNorth(CurrentLevel))-WorldMapLevelY(CurrentLevel))^2)
						WorldMapWalkingDX=Float(WorldMapLevelX(WorldMapLevelExitNorth(CurrentLevel))-WorldMapLevelX(CurrentLevel))/WorldMapWalking
						WorldMapWalkingDY=Float(WorldMapLevelY(WorldMapLevelExitNorth(CurrentLevel))-WorldMapLevelY(CurrentLevel))/WorldMapWalking
						CurrentLevelBackup=CurrentLevel
						CurrentLevel=WorldMapLevelExitNorth(CurrentLevel)
					EndIf
	
					
					CursorKeyPressed=True
				EndIf
		
			Else If (KeyDown(208) Or KeyDown(80)) ;down
				If CursorKeyPressed=False
					If WorldMapLevelExitSouth(CurrentLevel)>=0
						WorldMapWalking=100*Sqr((WorldMapLevelX(WorldMapLevelExitSouth(CurrentLevel))-WorldMapLevelX(CurrentLevel))^2+(WorldMapLevelY(WorldMapLevelExitSouth(CurrentLevel))-WorldMapLevelY(CurrentLevel))^2)
						WorldMapWalkingDX=Float(WorldMapLevelX(WorldMapLevelExitSouth(CurrentLevel))-WorldMapLevelX(CurrentLevel))/WorldMapWalking
						WorldMapWalkingDY=Float(WorldMapLevelY(WorldMapLevelExitSouth(CurrentLevel))-WorldMapLevelY(CurrentLevel))/WorldMapWalking
						CurrentLevelBackup=CurrentLevel
						CurrentLevel=WorldMapLevelExitSouth(CurrentLevel)
					EndIf
		
					
					CursorKeyPressed=True
				EndIf
			Else If (KeyDown(203) Or KeyDown(75)) ;left 
				If CursorKeyPressed=False
					If WorldMapLevelExitWest(CurrentLevel)>=0
						WorldMapWalking=100*Sqr((WorldMapLevelX(WorldMapLevelExitWest(CurrentLevel))-WorldMapLevelX(CurrentLevel))^2+(WorldMapLevelY(WorldMapLevelExitWest(CurrentLevel))-WorldMapLevelY(CurrentLevel))^2)
						WorldMapWalkingDX=Float(WorldMapLevelX(WorldMapLevelExitWest(CurrentLevel))-WorldMapLevelX(CurrentLevel))/WorldMapWalking
						WorldMapWalkingDY=Float(WorldMapLevelY(WorldMapLevelExitWest(CurrentLevel))-WorldMapLevelY(CurrentLevel))/WorldMapWalking
						CurrentLevelBackup=CurrentLevel
						CurrentLevel=WorldMapLevelExitWest(CurrentLevel)
					EndIf
	
					
					CursorKeyPressed=True
	
				EndIf
		
			Else If (KeyDown(205) Or KeyDown(77)) ;right
				If CursorKeyPressed=False
					If WorldMapLevelExitEast(CurrentLevel)>=0
						WorldMapWalking=100*Sqr((WorldMapLevelX(WorldMapLevelExitEast(CurrentLevel))-WorldMapLevelX(CurrentLevel))^2+(WorldMapLevelY(WorldMapLevelExitEast(CurrentLevel))-WorldMapLevelY(CurrentLevel))^2)
						WorldMapWalkingDX=Float(WorldMapLevelX(WorldMapLevelExitEast(CurrentLevel))-WorldMapLevelX(CurrentLevel))/WorldMapWalking
						WorldMapWalkingDY=Float(WorldMapLevelY(WorldMapLevelExitEast(CurrentLevel))-WorldMapLevelY(CurrentLevel))/WorldMapWalking
						CurrentLevelBackup=CurrentLevel
						CurrentLevel=WorldMapLevelExitEast(CurrentLevel)
					EndIf
	
					
					CursorKeyPressed=True
	
				EndIf
				
				
			
		
		
			
			Else If (KeyDown(28) Or KeyDown(156))
				If CursorKeyPressed=False
					
					CursorKeyPressed=True
					endmapscreen()
					StartLevel(currentlevel)
					
				EndIf
					
			Else If KeyDown(1)
				If CursorKeyPressed=False
					CursorKeyPressed=True
					endmapscreen()
					StartMenu(1)
					
		
				EndIf
			Else
				CursorKeyPressed=False
			EndIf
		EndIf
	EndIf
	
	


End Function
Function mapscreenupdate()
	LevelTimer=LevelTimer+1
	
	
	TurnEntity WorldMapCursor,0,2,0
	ScaleEntity WorldMapCursor,.10+.02*Sin(LevelTimer),.10+.02*Sin(LevelTimer),.10+.02*Sin(LevelTimer)

	If WorldMapWalking>0 
		WorldMapWalking=WorldMapWalking-1
		If WorldMapWalking>0
			TranslateEntity Camera,WorldMapWalkingDX,0,-WorldMapWalkingDY
			TranslateEntity WorldMapCursor,WorldMapWalkingDX,0,-WorldMapWalkingDY
		Else
			; reach destination
			PositionEntity Camera,WorldMapLevelX(CurrentLevel),3,-WorldMapLevelY(CurrentLevel)
			PositionEntity WorldMapCursor,WorldMapLevelX(CurrentLevel),0.02,-WorldMapLevelY(CurrentLevel)
			; secret level?
			If WorldMapLevelType(CurrentLevel)=1 And WorldMapLevelStatus(CurrentLevel)=3
				; reveal secret
				EndMapScreen()
				
				StartStoryScreen(0)
				WorldMapLevelStatus(CurrentLevel)=0
			
			; story/gate level?
			Else If worldMapleveltype(currentlevel)>10 
				EndMapScreen()
				
				StartStoryScreen(worldMapleveltype(currentlevel)-10)
				
		
			; passthrough level?
			Else If worldMapleveltype(currentlevel)=10
				If currentlevelbackup=WorldMapLevelExitNorth(CurrentLevel)
					Currentlevelbackup=currentlevel
					currentlevel=WorldMapLevelExitEast(CurrentLevel)
				Else
					Currentlevelbackup=currentlevel
					currentlevel=WorldMapLevelExitNorth(CurrentLevel)
				EndIf
				WorldMapWalking=100*Sqr((WorldMapLevelX(currentlevel)-WorldMapLevelX(CurrentLevelbackup))^2+(WorldMapLevelY(currentlevel)-WorldMapLevelY(CurrentLevelbackup))^2)
				WorldMapWalkingDX=Float(WorldMapLevelX(currentlevel)-WorldMapLevelX(CurrentLevelbackup))/WorldMapWalking
				WorldMapWalkingDY=Float(WorldMapLevelY(currentlevel)-WorldMapLevelY(CurrentLevelbackup))/WorldMapWalking
			Else
				SavePlayer()
			EndIf


		EndIf
	EndIf

End Function
Function endmapscreen()
	
	i=0
	;If Worldmap>0 FreeEntity Worldmap
	HideEntity Worldmap
	HideEntity WorldMapshade1
	HideEntity WorldMapshade2
		

	If WorldmapLevels>0 FreeEntity WorldmapLevels
	
	
	If WorldmapCursor>0 FreeEntity WorldmapCursor


	Repeat
		If worldmaptexturename$(i)<>"END"
			
			If WorldMapTexture(i)>0 
				FreeTexture WorldMapTexture(i)
				WorldMapTexture(i)=0
			EndIf
			
							
			
		EndIf
		i=i+1
	Until Worldmaptexturename$(i-1)="END"


End Function

Function StartStoryScreen(story)

		If story=6 Or story=9 Or story=10 Or story=21 Or story=24 Or story=25 Or story=30 Or story=35 Or story=44 Or story=45 Or story=46
			If story=6
				storymenupic(0)=1
			Else If story=9
				storymenupic(1)=1
			Else If story=10
				storymenupic(2)=1
			Else If story=21
				storymenupic(3)=1
			Else If story=24
				storymenupic(4)=1
			Else If story=25
				storymenupic(5)=1
			Else If story=30
				storymenupic(6)=1
			Else If story=35
				storymenupic(7)=1
			Else
				storymenupic(8)=1
			EndIf
			File=WriteFile("story.dat")
			If File=0
				Return
			EndIf

			For i=0 To 8
				WriteInt (File,storymenupic(i))
			Next
			CloseFile file
		EndIf

			

		If story >=44 And story <= 46 And storymenuselected=0
			If currentmusic<>1
				StopChannel Music
				music=PlayMusic("res4.dat") ; intro
				currentmusic=1
				ChannelVolume music,OptionsMusicLevel
			EndIf

			
		EndIf
			

		currentstory=story
		gamephase=0
		currentpage=1
		EndMenu() ; get rid of standard menu background
		CurrentMenu=7
		LevelTimer=10
		
		;first check if it's a gate that will be rejected:
		If StoryType(currentstory)=4
			If storygate(currentstory)<=20 
				If playergoldstars<storygate(currentstory)
					currentstory=currentstory+1
				EndIf
			Else If storygate(currentstory)<=40
				If playerrainbowstars<storygate(currentstory)-20
					currentstory=currentstory+1
				EndIf
			Else If storygate(currentstory)=41
				If playerpeeguespass=0 Then currentstory=currentstory+1
			Else If storygate(currentstory)=42
				If playerqookiesmap=0 Then currentstory=currentstory+1
			Else If storygate(currentstory)=43
				If playerforestmap=0 Then currentstory=currentstory+1
			EndIf
		EndIf

			
		
		; load the world map textures and create the map
		
		WorldMapTexture(14)=MyLoadTexture("graphics\parchment.bmp",1,False)
		ScaleTexture WorldMapTexture(14),.5,.5
		WorldMapLevels=CreatePlane()
		
		PositionEntity worldmaplevels,0,-.1,0
		RotateEntity worldmaplevels,0,0,0
		EntityTexture worldmaplevels,worldmaptexture(14)
		
		
		
	;	If version>=10 
			WorldMapTexture(15)=MyLoadTexture("graphics\"+StoryPicture$(currentstory),4,False)
	;	Else
	;		WorldMapTexture(15)=MyLoadTexture("graphics\"+Left$(StoryPicture$(currentstory),Len(storypicture$(currentstory))-4)+".jpg",4,False)
;
;		EndIf
		EntityTexture StoryScreen,WorldMapTexture(15)
		
		ShowEntity StoryScreen
		
			
		
		
		
		;Worldmap2=CopyEntity(WorldMap)
		;TranslateEntity Worldmap2,.1001,.001,.1001
						
					
						
		RotateEntity Camera,90,0,0
		PositionEntity Camera,2,3.94,-1.5		
						
		UpdateWorld
		
		MenuBrightnessGoal=254
		
		CurrentStoryChoice=0
		
		LightColor Levellight,255,255,255
		AmbientLight 255,255,255

			

End Function



Function storyscreenloop()

	
	; display the story
	letters=leveltimer/5
	
	For i=0 To 10
		centerText("",1,1,50+i)
	Next
	
	For i=1 To 5
		If Len(StoryText$(CurrentStory,currentpage,i))<=letters
			CenterText(StoryText$(currentstory,Currentpage,i),12+i,1,55+i)
			letters=letters-Len(StoryText$(currentstory,Currentpage,i))
		Else If letters>0
		
			DisplayText(Left$(StoryText$(currentstory,Currentpage,i),letters),25-0.5*Len(StoryText$(currentstory,Currentpage,i)),12+i,1,55+i)
			letters=0
		EndIf
	Next
	
	; display menu or enterkey 
	If letters>0 ;only once the story has been spelled out
		
		;gate menu
		If Storytype(Currentstory)=4 And currentpage=storypages(currentstory)
			If currentstoryChoice=0
				CenterText("*** YES, PLEASE ***",16,1,50)
				CenterText("NOT YET",17,1,51)
			Else
				CenterText("YES, PLEASE",16,1,50)
				CenterText("*** NOT YET ***",17,1,51)
			EndIf
			CenterText("(Use Arrow Keys and Enter To Select)",18,1,52)
		Else
			CenterText("(Press Enter/Return to continue)",18,1,50)
		EndIf	
	
	EndIf
	

	
	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next



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

		
		; do the updates in here
		
		LevelTimer=LevelTimer+1
			
		storyscreenupdate()
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	
	
	Flip

	
	; Input Etc
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False
			
			currentStoryChoice=1-currentStoryChoice
			
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) 
		If CursorKeyPressed=False
			currentStoryChoice=1-currentStoryChoice
			CursorKeyPressed=True
		EndIf
	Else If (KeyDown(203) Or KeyDown(75)) 
		If CursorKeyPressed=False
			currentStoryChoice=1-currentStoryChoice

			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(205) Or KeyDown(77)) 
		If CursorKeyPressed=False
			currentStoryChoice=1-currentStoryChoice

			CursorKeyPressed=True
		EndIf
		
	
	
	Else If (KeyDown(28) Or KeyDown(156)); Or KeyDown(1)) do separately and always reject
		If CursorKeyPressed=False
			
			CursorKeyPressed=True
			If letters>0
			; based on type:
			If currentpage<storypages(currentstory)
				;next page
				currentpage=currentpage+1
				leveltimer=5
			Else
				If storymenuselected=1
					If currentstory<5
						; next story
						endstoryscreen()
						currentstory=currentstory+1
						startstoryscreen(currentstory)
					Else

						endstoryscreen()
						startmenu(18)
						storymenuselected=0
						Return
					EndIf
				Else If storytype(currentstory)=0
					; back to map
					endstoryscreen()
					
					; unless it's story 5 - then start level 1
					If currentstory=5
						currentlevel=1
						Saveplayer()
						StartLevel(1)
					Else If currentstory=45 Or currentstory=46
						; or if it's ending #2/#3
						StartEndingSequence()
					
					Else
						startmapscreen()
					EndIf
				Else If storytype(currentstory)=1
					; next story
					endstoryscreen()
					currentstory=currentstory+1
					startstoryscreen(currentstory)
				Else If storytype(currentstory)=2
					; continue forward/turn off story
					If version<10 And currentstory=9
						; it's the demo end
						endstoryscreen()
						StartMenu(11)
					Else
						worldMapleveltype(currentlevel)=10
						If currentlevelbackup=WorldMapLevelExitNorth(CurrentLevel)
							Currentlevelbackup=currentlevel
							currentlevel=WorldMapLevelExitEast(CurrentLevel)
						Else
							Currentlevelbackup=currentlevel
							currentlevel=WorldMapLevelExitNorth(CurrentLevel)
						EndIf
						WorldMapWalking=100*Sqr((WorldMapLevelX(currentlevel)-WorldMapLevelX(CurrentLevelbackup))^2+(WorldMapLevelY(currentlevel)-WorldMapLevelY(CurrentLevelbackup))^2)
						WorldMapWalkingDX=Float(WorldMapLevelX(currentlevel)-WorldMapLevelX(CurrentLevelbackup))/WorldMapWalking
						WorldMapWalkingDY=Float(WorldMapLevelY(currentlevel)-WorldMapLevelY(CurrentLevelbackup))/WorldMapWalking
		
						endstoryscreen()
						startmapscreen()
					EndIf
				Else If storytype(currentstory)=3 Or (storytype(currentstory)=4 And currentstorychoice=1)
					; turned back
					ex=CurrentLevelBackup
					Currentlevelbackup=currentlevel
					currentlevel=ex
					
					WorldMapWalking=100*Sqr((WorldMapLevelX(currentlevel)-WorldMapLevelX(CurrentLevelbackup))^2+(WorldMapLevelY(currentlevel)-WorldMapLevelY(CurrentLevelbackup))^2)
					WorldMapWalkingDX=Float(WorldMapLevelX(currentlevel)-WorldMapLevelX(CurrentLevelbackup))/WorldMapWalking
					WorldMapWalkingDY=Float(WorldMapLevelY(currentlevel)-WorldMapLevelY(CurrentLevelbackup))/WorldMapWalking
	
					endstoryscreen()
					startmapscreen()
				Else If storytype(currentstory)=4 And currentstorychoice=0
					; pay up and go forward
					If storygate(currentstory)<=20 
						playergoldstars=playergoldstars-storygate(currentstory)
					Else If storygate(currentstory)<=40
						playerrainbowstars=playerrainbowstars-storygate(currentstory)+20
					Else If storygate(currentstory)=41
						playerpeeguespass=0
					Else If storygate(currentstory)=42
						playerqookiesmap=0 
					Else If storygate(currentstory)=43
						playerforestmap=0 
					EndIf

					
					; forward
					worldMapleveltype(currentlevel)=10
					If currentlevelbackup=WorldMapLevelExitNorth(CurrentLevel)
						Currentlevelbackup=currentlevel
						currentlevel=WorldMapLevelExitEast(CurrentLevel)
					Else
						Currentlevelbackup=currentlevel
						currentlevel=WorldMapLevelExitNorth(CurrentLevel)
					EndIf
					WorldMapWalking=100*Sqr((WorldMapLevelX(currentlevel)-WorldMapLevelX(CurrentLevelbackup))^2+(WorldMapLevelY(currentlevel)-WorldMapLevelY(CurrentLevelbackup))^2)
					WorldMapWalkingDX=Float(WorldMapLevelX(currentlevel)-WorldMapLevelX(CurrentLevelbackup))/WorldMapWalking
					WorldMapWalkingDY=Float(WorldMapLevelY(currentlevel)-WorldMapLevelY(CurrentLevelbackup))/WorldMapWalking
	
					endstoryscreen()
					startmapscreen()
					

				EndIf
			  
			EndIf
			EndIf		

			
			
			
			
			
			
			
		EndIf
	
	
	Else
		CursorKeyPressed=False
	EndIf
	
	


End Function
Function storyscreenupdate()
	LevelTimer=LevelTimer+1
	
End Function

Function endstoryscreen()
	For i=0 To 15
		centertext("",18,1,50+i)
	Next
	If storyscreen>0 HideEntity storyscreen
	If Worldmaplevels>0 FreeEntity Worldmaplevels

	For i=14 To 15
		
			
		If WorldMapTexture(i)>0 
			FreeTexture WorldMapTexture(i)
			WorldMapTexture(i)=0
		EndIf
			
							
	Next


End Function


Function startendingsequence()

	

	If currentmusic<>5
		StopChannel Music
		music=PlayMusic("res7.dat") ; slow
		currentmusic=5
	EndIf
	ChannelVolume music,OptionsMusicLevel

	RotateEntity Camera,0,-90,0
	TurnEntity camera,12,0,0
	PositionEntity Camera,0,6,0		
						
	UpdateWorld

	leveltimer=1
	currentmenu=16
	
	
	FloorTexture = MyLoadTexture ("graphics\castlefloor1.jpg",1,False)
	FloorTexture2 = MyLoadTexture ("graphics\castlefloor2.jpg",1,False)
	WallTextureSide1a = MyLoadTexture ("graphics\castlewallside1a.jpg",1,False)
	ScaleTexture walltextureside1a,.0150,.3
	BackGroundTexture=MyLoadTexture ("graphics\eveningsky.jpg",1,False)
	ScaleTexture BackGroundTexture,.01,.15
	Stinkertexture2 = myloadtexture("graphics\stinker2.bmp",1,False)
	
	FreeEntity ItemMesh(1)
	ItemMesh(1)=MyLoadMesh("graphics\goldstar.3ds",False)
	RotateMesh Itemmesh(1),-90,0,0
	;ScaleEntity ItemMesh(1),0.5,0.5,0.5
	EntityTexture ItemMesh(1),RainbowTexture
	HideEntity ItemMesh(1)

	
	;floor
	boxentity(0)=CreatePlane()
	EntityTexture boxentity(0),floortexture
	
	;citywall
	boxentity(1)=CreateCylinder(64,0)
	FlipMesh boxentity(1)
	ScaleEntity boxentity(1),60,3,60
	EntityTexture boxentity(1),walltextureside1a	
	
	;heroes
	For i=1 To 4
		boxentity(1+i)=CopyEntity(playermesh(i))
		EntityTexture boxentity(1+i),playertexture(i)
		PositionEntity boxentity(1+i),-5+i*2,0,10
		ScaleEntity boxentity(1+i),1.5,1.5,1.5

	Next
	ScaleEntity boxentity(3),1.1,1.1,1.1
	
	;sky
	boxentity(6)=CreateCylinder(64,0)
	FlipMesh boxentity(6)
	TranslateEntity boxentity(6),0,-4,0
	ScaleEntity boxentity(6),70,60,70
	EntityTexture boxentity(6),backgroundtexture	

	
	;houses
	For i=0 To 3
		j=7+i
		boxentity(j)=CopyMesh(pillarmesh(8))
		EntityTexture boxentity(j),housetexture(Rand(1,4))
		PositionEntity boxentity(j),10+i*10,0,20
		RotateEntity boxentity(j),0,90,0
		ScaleEntity boxentity(j),2,2,2
	Next
	j=11
	boxentity(j)=CopyMesh(pillarmesh(8))
		EntityTexture boxentity(j),housetexture(Rand(1,4))
		PositionEntity boxentity(j),30,0,40
		RotateEntity boxentity(j),0,0,0
		ScaleEntity boxentity(j),2,2,2

	j=12
	boxentity(j)=CopyMesh(pillarmesh(8))
		EntityTexture boxentity(j),housetexture(Rand(1,4))
		PositionEntity boxentity(j),-20,0,40
		RotateEntity boxentity(j),0,90,0
		ScaleEntity boxentity(j),2,2,2

	For i=0 To 3
		j=13+i
		boxentity(j)=CopyMesh(pillarmesh(8))
		EntityTexture boxentity(j),housetexture(Rand(1,4))
		PositionEntity boxentity(j),40,0,i*-10
		RotateEntity boxentity(j),0,0,0
		ScaleEntity boxentity(j),2,2,2

	Next
	For i=0 To 2
		j=17+i
		boxentity(j)=CopyMesh(pillarmesh(8))
		EntityTexture boxentity(j),housetexture(Rand(1,4))
		PositionEntity boxentity(j),10+i*10,0,-40
		RotateEntity boxentity(j),0,-90,0
		ScaleEntity boxentity(j),2,2,2

	Next
	For i=0 To 3
		j=20+i
		boxentity(j)=CopyMesh(pillarmesh(8))
		EntityTexture boxentity(j),housetexture(Rand(1,4))
		PositionEntity boxentity(j),-10,0,-10-i*10
		RotateEntity boxentity(j),0,180,0
		ScaleEntity boxentity(j),2,2,2

	Next
	For i=0 To 1
		j=24+i
		boxentity(j)=CopyMesh(pillarmesh(8))
		EntityTexture boxentity(j),housetexture(Rand(1,4))
		PositionEntity boxentity(j),-30+i*10,0,-10
		RotateEntity boxentity(j),0,-90,0
		ScaleEntity boxentity(j),2,2,2

	Next
	j=26
		boxentity(j)=CopyMesh(pillarmesh(8))
		EntityTexture boxentity(j),housetexture(Rand(1,4))
		PositionEntity boxentity(j),-50,0,0
		RotateEntity boxentity(j),0,180,0
		ScaleEntity boxentity(j),2,2,2
	For i=0 To 2
		j=27+i
		boxentity(j)=CopyMesh(pillarmesh(8))
		EntityTexture boxentity(j),housetexture(Rand(1,4))
		PositionEntity boxentity(j),-40,0,10+10*i
		RotateEntity boxentity(j),0,180,0
		ScaleEntity boxentity(j),2,2,2

	Next
	j=30
		boxentity(j)=CopyMesh(pillarmesh(8))
		EntityTexture boxentity(j),housetexture(Rand(1,4))
		PositionEntity boxentity(j),-30,0,40
		RotateEntity boxentity(j),0,90,0
		ScaleEntity boxentity(j),2,2,2
	j=31
		boxentity(j)=CopyMesh(pillarmesh(8))
		EntityTexture boxentity(j),housetexture(Rand(1,4))
		PositionEntity boxentity(j),-20,0,10
		RotateEntity boxentity(j),0,90,0
		ScaleEntity boxentity(j),2,2,2

	; pond
	boxentity(32)=CreateCylinder(64,0)
	ScaleEntity boxentity(32),10.1,0.7,10.1
	PositionEntity boxentity(32),20,0,-20
	EntityTexture boxentity(32),walltextureside1a	
	boxentity(33)=CreateCylinder(64,0)
	ScaleEntity boxentity(33),10,0.7,10
	PositionEntity boxentity(33),20,0,-20
	EntityTexture boxentity(33),walltextureside1a	
	FlipMesh boxentity(33)
	boxentity(34)=CreateCylinder(64,1)
	ScaleEntity boxentity(34),9.9,0.4,9.9
	PositionEntity boxentity(34),20,0,-20
	EntityTexture boxentity(34),watertexture
	;fish
	For i=0 To 4
		boxentity(35+i)=CopyEntity(fishmesh)
		
		PositionMesh boxentity(35+i),0,Rnd(.4,1.0),0
		PositionEntity boxentity(35+i),Rand(17,23),-.2,Rand(-17,-23)
		RotateEntity boxentity(35+i),0,Rand(0,360),0
		TurnEntity boxentity(35+i),0,0,Rand(0,360)
	Next
	
	boxentity(40)=CreatePivot ()
	PositionEntity boxentity(40),0,0,40
	boxentity(41)=CreatePivot()
	
	; stinkers in front of heroes
	
	For i=0 To 9
		boxentity(42+i)=CopyEntity(stinkermesh)
		ScaleEntity boxentity(42+i),1.3,1.3,1.3
		EntityTexture boxentity(42+i),stinkertexture2
		PositionEntity boxentity(42+i),-5+(((i+1)*7) Mod 10),0,15+i*2
		PointEntity boxentity(42+i),boxentity(40)
	Next
	
	; stinker jumping
	boxentity(52)=CreatePivot()
	PositionEntity boxentity(52),25,0,5
	For i=0 To 9
		boxentity(53+i)=CopyEntity(stinkermesh)
		ScaleEntity boxentity(53+i),1.5,1.5,1.5
		EntityTexture boxentity(53+i),stinkertexture2
		PositionEntity boxentity(53+i),25+7*Sin(i*36),0,5+7*Cos(i*36)
		PointEntity boxentity(53+i),boxentity(52)
		AnimateMD2 boxentity(53+i),2,0.4,36,42
	Next
	
	; rainbow star
	boxentity(63)=CopyEntity(itemmesh(1))
	PositionEntity boxentity(63),0,5,40
	ScaleEntity boxentity(63),5,5,5
	
	boxentity(64)=CopyEntity(itemmesh(1))
	PositionEntity boxentity(64),25,4,5
	ScaleEntity boxentity(64),2,2,2

	
	; coilygate
	boxentity(65)=CreateCylinder(64,0)
	ScaleEntity boxentity(65),7.1,2,7.1
	PositionEntity boxentity(65),-25,0,25
	EntityTexture boxentity(65),walltextureside1a	
	boxentity(66)=CreateCylinder(64,0)
	ScaleEntity boxentity(66),7,2,7
	PositionEntity boxentity(66),-25,0,25
	EntityTexture boxentity(66),walltextureside1a	
	FlipMesh boxentity(66)
	; coilys
	For i=0 To 4
		boxentity(67+i)=CopyEntity(monstermesh(0))
		ScaleEntity boxentity(67+i),1,1,1
	Next
	
	; gold stars
	For i=0 To 9
		boxentity(72+i)=CopyEntity(itemmesh(1))
		ScaleEntity boxentity(72+i),2,2,2
		EntityTexture boxentity(72+i),GoldstarTexture
	Next

	boxentity(82)=CopyEntity(stinkermesh)
	ScaleEntity boxentity(82),1.2,1.2,1.2
	EntityTexture boxentity(82),stinkertexture2
	PositionEntity boxentity(82),0,0,-20
	;PointEntity boxentity(82),boxentity(40)
	
	For i=83 To 120
		boxentity(i)=CreateCube()
		ScaleEntity boxentity(i),.3,.3,0.01
		EntityTexture boxEntity(i),ParticleTexture,3
	Next


	
	

	
		
	


	
	
	
	
	

	


End Function

Function endingupdate()

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
	
	
	; text in here
	
	

	If LevelTimer<1000
		centertext("",1,1,52)
		centertext("",2,1,53)
		centertext("",3,1,50)
	Else If LevelTimer<2000
		centertext("",1,1,52)
		centertext("WONDERLAND",1.5,1,53)
		centertext("Secret Worlds",2.5,1,50)
	Else If LevelTimer<3000
		centertext("A",1,1,52)
		centertext("Midnight Synergy",2,1,53)
		centertext("Game",3,1,50)
	Else If LevelTimer<4000
		centertext("DESIGN, CODE, AUDIOVISUALS",1,1,52)
		centertext("Patrick Maidorn",2,1,53)
		centertext("",3,1,50)
	Else If LevelTimer<5000
		centertext("LEVEL DESIGNERS",1,1,52)
		centertext("Malcolm Fowler",2,1,53)
		centertext("Janet Galle",3,1,50)
	Else If LevelTimer<6000
		centertext("LEVEL DESIGNERS",1,1,52)
		centertext("Carolyn Kinney",2,1,53)
		centertext("Dean J. Lemon",3,1,50)
	Else If LevelTimer<7000
		centertext("LEVEL DESIGNERS",1,1,52)
		centertext("Julie Majewski",2,1,53)
		centertext("Roger Melin",3,1,50)
	Else If LevelTimer<8000
		centertext("LEVEL DESIGNERS",1,1,52)
		centertext("Pauline Robinson",2,1,53)
		centertext("Carolyn Schindler",3,1,50)
	Else If LevelTimer<9000
		centertext("LEVEL DESIGNERS",1,1,52)
		centertext("Steve Seaman",2,1,53)
		centertext("Mette Sidelmann",3,1,50)
	Else If LevelTimer<10000
		centertext("STORY ILLUSTRATIONS",1.5,1,52)
		centertext("James Abraham",2.5,1,53)
		centertext("",3,1,50)
	Else If LevelTimer<11000
		centertext("MUSIC",1.5,1,52)
		centertext("Jonne Valtonen",2.5,1,53)
		centertext("",3,1,50)
	Else If LevelTimer<12000
		centertext("ADDITIONAL 3D GRAPHICS",1,1,52)
		centertext("Danny Hawrysio",2,1,53)
		centertext("Dean J. Lemon",3,1,50)
	Else If LevelTimer<13000
		centertext("ADDITIONAL 3D GRAPHICS",1,1,52)
		centertext("Pauline Robinson",2,1,53)
		centertext("Steve Seaman",3,1,50)
	Else If LevelTimer<14000
		centertext("TESTERS",1,1,52)
		centertext("Bearsland, carolyn, Maureen Davidson",2,1,53)
		centertext("Gen, jperleevt, Kaboom!",3,1,50)
	Else If LevelTimer<15000
		centertext("TESTERS",1,1,52)
		centertext("Lillian, Lori, Pat",2,1,53)
		centertext("Ramonda, Robbie",3,1,50)
	Else If LevelTimer<16000
		centertext("A Big THANK YOU To",1.5,1,52)
		centertext("The Usual Suspects",2.5,1,53)
		centertext("",3,1,50)

	Else If LevelTimer<19000
		centertext("THE END",1,1,52)
		centertext("",2,1,53)
		centertext("Thank you for playing!",3,1,50)
	Else 
		If currentstory=45
			centertext("Now can you find every last",1,1,52)
			centertext("Bonus Coin before returning",2,1,53)
			centertext("to rescue the Stinkers again?",3,1,50)
		Else
			centertext("Congratulations on a Perfect Score!",1,1,52)
			centertext("Connect to the Internet and visit",2,1,53)		
			centertext("TEXT REDACTED IN OPEN SOURCE VERSION",3,1,50)
		EndIf
	EndIf


;	centertext(Str$(leveltimer),5,1,54)



	


	
		
	
	
	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next

	
	
	
	For k=1 To TweenTicks
		Tweentime=Tweentime+Tweenperiod
		If k=Tweenticks 
			CaptureWorld
			
		EndIf

		LevelTimer=LevelTimer+1
		If leveltimer=28800
			leveltimer=0
		EndIf
		
		endingupdate2()
		
		

			
		
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	
	Flip
	
	
	If KeyHit(1) And cursorkeypressed=False
		endendingsequence()
		startmapscreen()
		cursorkeypressed=True
	Else
		CursorKeyPressed=False
	End If


	


End Function

Function endingupdate2()
	TurnEntity camera,-12,0,0
	TurnEntity Camera,0,.05,0
	TurnEntity camera,12,0,0
	;turn heroes / stinkers
	If (leveltimer Mod 3600)=0
		For i=2 To 5
			If Rand(0,100)<20
				PointEntity boxentity(i),boxentity(41)
				AnimateMD2 boxentity(i),1,0.05+Rnd(0.01,0.1),44,52
			Else 
				RotateEntity boxentity(i),0,0,0
				AnimateMD2 boxentity(i),0,1,0,0
			EndIf
		Next
		
		For i=42 To 51
			If Rand(0,100)<40
				PointEntity boxentity(i),boxentity(41)
				AnimateMD2 boxentity(i),1,0.05+Rnd(0.01,0.1),44,52
			Else 
				PointEntity boxentity(i),boxentity(40)
				AnimateMD2 boxentity(i),0,1,0,0
			EndIf
		Next

	EndIf
	;jump fish
	For i=0 To 4
		TurnEntity boxentity(35+i),0,0,1+i*.25
	Next
	; turn star
	TurnEntity boxentity(63),0,1,0
	TurnEntity boxentity(64),0,-3,0

	;jumping stinkers
	For i=53 To 62
		PositionEntity boxentity(i),25+7*Sin((i-53)*36),Sin((leveltimer*5 +i*36)Mod 180),5+7*Cos((i-53)*36)
	Next
	For i=67 To 71
		PositionEntity boxentity(i),-25+4*Sin((i-67)*50),Sin((leveltimer*5 +i*36)Mod 180),25+4*Cos((i-67)*50)
		TurnEntity boxentity(i),0,i/2 - 32,0
	Next
	For i=72 To 81
		PositionEntity boxentity(i),5*Sin((i-72)*36),1+3*Sin((leveltimer +i*36)Mod 360),-40+5*Cos((i-72)*36)
		TurnEntity boxentity(i),0,i/2 - 35,0
	Next
	For i=83 To 100
		RotateEntity  boxentity(i),0,0,(leveltimer*2+i*5) Mod 360
	;	PositionEntity boxentity(i),40*Sin(7.0*((leveltimer/100.0) Mod 360)+i*8),4.5+2*Sin(((leveltimer) Mod 360)+i*20),-40*Cos(11.0*((leveltimer/100.0) Mod 360)+i*8)
	;	PositionEntity boxentity(i),-40+i/30.0*Sin(leveltimer+i*20),3+2.5*Sin(leveltimer*2+i*40),i/30.0*Cos(leveltimer+i*20)
		PositionEntity boxentity(i),-55*Sin(leveltimer/5.0+i*5),7+3*Cos(leveltimer+i*15),-55*Cos(leveltimer/5.0+i*5)
	Next
	For i=101 To 120
		RotateEntity  boxentity(i),0,0,(leveltimer*2+i*5) Mod 360
	;	PositionEntity boxentity(i),40*Sin(7.0*((leveltimer/100.0) Mod 360)+i*8),4.5+2*Sin(((leveltimer) Mod 360)+i*20),-40*Cos(11.0*((leveltimer/100.0) Mod 360)+i*8)
		PositionEntity boxentity(i),4*Cos(leveltimer/2+i*18),5+4*Sin(leveltimer/2+i*18),45+10*Cos(leveltimer)
	Next

	

	

	

		
End Function

Function endendingsequence()

	FreeTexture FloorTexture
	FreeTexture FloorTexture2 
	FreeTexture WallTextureSide1a 
	FreeTexture backgroundtexture

	For i=0 To 120
		FreeEntity boxentity(i)
	Next


End Function