Function MenuSetup()

	MenuBackGroundTex=MyLoadTexture("res20.dat",1,False)
	ScaleTexture MenuBackGroundTex,20,20
	
	MenuBackGround = CreatePlane()
	RotateEntity MenuBackGround,-45,0,0
	PositionEntity MenuBackGround,0,0,20
	EntityTexture MenuBackGround,MenuBackGroundTex	
	
	MenuBackGround2 = CreatePlane()
	RotateEntity MenuBackGround2,-45,0,0
	PositionEntity MenuBackGround2,0,0,10
	EntityTexture MenuBackGround2,MenuBackGroundTex	
	EntityAlpha MenuBackGround2,.5
	
	Logotex1=MyLoadTexture("graphics\wonderlanda.bmp",4,False)
	Logo1 = CreateFlatSquare(logotex1)
	RotateEntity Logo1,-90,0,0
;	ScaleEntity Logo1,1.5,1.5,1.5
	
	Logotex2=MyLoadTexture("graphics\wonderlandb.bmp",4,False)
	Logo2 = CreateFlatSquare(logotex2)
	RotateEntity Logo2,-90,0,0
;	ScaleEntity Logo2,1.5,1.5,1.5

	

	
	
	
	
	HideEntity MenuBackGround
	HideEntity MenuBackGround2
	
	MenuSelector = CreateCube (Camera)
	ScaleEntity MenuSelector,1,1,0.001
	MenuSelectortex = MyLoadTexture("graphics\menubar.jpg",1,False)
	ScaleEntity MenuSelector,.35,.02,0.001
	TranslateEntity MenuSelector,-.5+20*.02+.01,0.375-(-2000)*.04-.02,0
	EntityTexture MenuSelector,MenuSelectortex
	EntityAlpha MenuSelector,.6
	
	WorldLevelTexture=MyLoadTexture("graphics\levelsigns2.bmp",4,False)
	WorldLevelTexture2=MyLoadTexture("graphics\levelsigns2b.bmp",4,False)


End Function



Function StartMenu(Menu)

	; 0-Test Title Screen with Level Selector
	; 1-Main Menu
	; 2-Player Select Screen
	; 3-Enter New Player Name
	; 4-Level Select Screen
	; 5-In Game Menu
	For i=0 To 9
		If TitleChar(i)<>0
			FreeEntity TitleChar(i)
			TitleChar(i)=0
		EndIf
	Next 
	
	If Menu=0; Intro PreAmble
		; Camera
		RotateEntity Camera,0,0,0
		PositionEntity Camera,0,0,-4
		
		; Lights
		MenuBackGroundTex=MyLoadTexture("res20.dat",1,False)
		ScaleTexture MenuBackGroundTex,20,20
		EntityTexture MenuBackGround,MenuBackGroundTex
		MenuBackGroundTex2=MyLoadTexture("res21.dat",1,False)
		ScaleTexture MenuBackGroundTex2,20,20
		EntityTexture MenuBackGround2,MenuBackGroundTex2
		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255	

		; Music
		;If ChannelPlaying(Music)=True
		;	StopChannel(Music)
		;EndIf
		
		; BackGround
		ShowEntity MenuBackGround
		ShowEntity MenuBackGround2
		ShowEntity MenuSelector
		HideEntity Logo1
		HideEntity Logo2

	
	Else If Menu=1	; Start Menu
		If IntroVoice=False
			PlayMySound(SFXTalk(5),0)
			Delay 1000
			IntroVoice=True
		EndIf
		
		; Camera
		RotateEntity Camera,0,0,0
		PositionEntity Camera,0,0,-4
		
		; Lights
		MenuBackGroundTex=MyLoadTexture("res20.dat",1,False)
		ScaleTexture MenuBackGroundTex,20,20
		EntityTexture MenuBackGround,MenuBackGroundTex
		MenuBackGroundTex2=MyLoadTexture("res21.dat",1,False)
		ScaleTexture MenuBackGroundTex2,20,20
		EntityTexture MenuBackGround2,MenuBackGroundTex2
	

		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255	

		
		
		; BackGround
		ShowEntity MenuBackGround
		ShowEntity MenuBackGround2
		ShowEntity MenuSelector
		ShowEntity Logo1
		ShowEntity Logo2
		
		PositionEntity Logo1,-.99,1.52,0
		EntityAlpha Logo1,0
		PositionEntity Logo2,0,1.52,0
		EntityAlpha Logo2,0
		
		; Music
		If DoesSoundExist=True And CurrentMenu<>7 And CurrentMenu<>9 And CurrentMenu<>10
			StopChannel(Music)
			Music=PlayMusic("res4.dat")
			ChannelVolume music,OptionsMusicLevel
		EndIf

		; Characters
		TitleChar(0)=CopyEntity(StinkyMesh)
		EntityTexture TitleChar(0),StinkyTexture
		RotateEntity TitleChar(0),45,190,0
		ScaleEntity TitleChar(0),.05,.05,.05
		PositionEntity TitleChar(0),-.86,.4,0
		AnimateMD2 TitleChar(0),1,0.15,44,52
		EntityAlpha TitleChar(0),0
		
		TitleChar(1)=CopyEntity(LoofMesh)
		EntityTexture TitleChar(1),LoofTexture
		RotateEntity TitleChar(1),45,170,0
		ScaleEntity TitleChar(1),.04,.04,.04
		PositionEntity TitleChar(1),.86,.4,0
		AnimateMD2 TitleChar(1),1,0.25,53,61
		EntityAlpha TitleChar(1),0
		
		TitleChar(2)=CopyEntity(CannonMesh)
		;EntityTexture TitleChar(2),CannonTexture
		RotateEntity TitleChar(2),-90,90,20
		ScaleEntity TitleChar(2),.045,.045,.045
		PositionEntity TitleChar(2),-1.2,.4,0
		EntityAlpha TitleChar(2),0
		
		TitleChar(3)=CopyEntity(MonsterMesh(0))
		;EntityTexture TitleChar(3),CannonTexture
		RotateEntity TitleChar(3),0,90,0
		ScaleEntity TitleChar(3),.02,.02,.02
		PositionEntity TitleChar(3),-.4,.4,0
		EntityAlpha TitleChar(3),0
		
		TitleChar(4)=CopyEntity(MonsterMesh(3))
		;EntityTexture TitleChar(3),CannonTexture
		RotateEntity TitleChar(4),0,180,0
		ScaleEntity TitleChar(4),.045,.045,.045
		PositionEntity TitleChar(4),.4,.4,0
		AnimateMD2 TitleChar(4),1,.4,1,24
		EntityAlpha TitleChar(4),0

		TitleChar(5)=CopyEntity(MonsterMesh(1))
		;EntityTexture TitleChar(3),CannonTexture
		RotateEntity TitleChar(5),-90,0,90
		ScaleEntity TitleChar(5),.025,.025,.025
		PositionEntity TitleChar(5),1.2,.4,0
		EntityAlpha TitleChar(5),0





		
		
		LevelTimer=0

	Else If Menu=2	; Player Select Screen
		; Camera
		RotateEntity Camera,0,0,0
		PositionEntity Camera,0,0,-4
		
		; Lights
		MenuBackGroundTex=MyLoadTexture("res20.dat",1,False)
		ScaleTexture MenuBackGroundTex,20,20
		EntityTexture MenuBackGround,MenuBackGroundTex
		MenuBackGroundTex2=MyLoadTexture("res21.dat",1,False)
		ScaleTexture MenuBackGroundTex2,20,20
		EntityTexture MenuBackGround2,MenuBackGroundTex2
		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255	
		
		; Music
		If DoesSoundExist=True
			If ChannelPlaying(Music)=True
				StopChannel(Music)
			EndIf
		EndIf
		
		; BackGround
		ShowEntity MenuBackGround
		ShowEntity MenuBackGround2
		ShowEntity MenuSelector
		HideEntity Logo1
		HideEntity Logo2
	
				
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
			ex$=ReadString$(File)
			ix=ReadInt(File)
			PlayerLevels(i)=ReadInt(File)
			PLayerScore(i)=ReadInt(File)
			CloseFile(File)
		Next

	
	Else If Menu=3; Enter New Player Name
		; Camera
		RotateEntity Camera,0,0,0
		PositionEntity Camera,0,0,-4
		
		; Lights
		MenuBackGroundTex=MyLoadTexture("res20.dat",1,False)
		ScaleTexture MenuBackGroundTex,20,20
		EntityTexture MenuBackGround,MenuBackGroundTex
		MenuBackGroundTex2=MyLoadTexture("res21.dat",1,False)
		ScaleTexture MenuBackGroundTex2,20,20
		EntityTexture MenuBackGround2,MenuBackGroundTex2
		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255	


		; Music
		If DoesSoundExist=True
			If ChannelPlaying(Music)=True
				StopChannel(Music)
			EndIf
		EndIf

		
		; BackGround
		ShowEntity MenuBackGround
		ShowEntity MenuBackGround2
		ShowEntity MenuSelector
		HideEntity Logo1
		HideEntity Logo2
	
				
		; Special Stuff
		CurrentPlayerName$=""	
	
	Else If Menu=4; Level Selector
		; Camera
		RotateEntity Camera,0,0,0
		PositionEntity Camera,0,0,-4
		
		; Lights
		MenuBackGroundTex=MyLoadTexture("res20.dat",1,False)
		ScaleTexture MenuBackGroundTex,20,20
		EntityTexture MenuBackGround,MenuBackGroundTex
		MenuBackGroundTex2=MyLoadTexture("res21.dat",1,False)
		ScaleTexture MenuBackGroundTex2,20,20
		EntityTexture MenuBackGround2,MenuBackGroundTex2
		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255	

		
		; Music
		If CurrentMenu<4 And DoesSoundExist=True
			StartLevelMusic()
		EndIf

		
		; BackGround
		ShowEntity MenuBackGround
		ShowEntity MenuBackGround2
		HideEntity MenuSelector
		HideEntity Logo1
		HideEntity Logo2

		LevelTimer=0
		
		; Special Stuff
		
		; add the squares for the levels
		; how many levels?
		CurrentWorldLevelN=0
		Repeat
			If WorldLevelName$(CurrentWorld*10+CurrentWorldLevelN)="" Then Exit
			CurrentWorldLevelN=CurrentWorldLevelN+1
		Until CurrentWorldLevelN=10
		
		
		; create the entities
		NextWorldFlag=0
		For j=0 To CurrentWorld
			For i=0 To 9
				If WorldLevelName$(j*10+i)<>"" And CurrentPlayerLevelScore(j*10+i)=0 Then NextWorldFlag=1
			Next
		Next

		For i=0 To CurrentWorldLevelN+Sgn(CurrentWorld)
		
			; create the flat squares for each level
			
			
			CurrentWorldLevelEntity(i)=CreateMesh()
			Brush=CreateBrush()
			BrushTexture Brush,WorldLevelTexture
			Surface=CreateSurface(CurrentWorldLevelEntity(i),Brush)

			If i=CurrentWorldLevelN
				; forward level
				; open
				If NextWorldFlag=0 Or (CurrentWorld<9 And LockWorlds=1)
					AddVertex(Surface,-1,0,1, .5,-.5)
					AddVertex(Surface,1,0,1, 1,-.5)
					AddVertex(Surface,1,0,-1, 1,0)
					AddVertex(Surface,-1,0,-1, .5,0)
					If Language=0
						CurrentWorldLevelName$(i,0)="The Gateway Is Open"
						CurrentWorldLevelName$(i,1)="Press 'Enter' To Continue"
						CurrentWorldLevelName$(i,2)=""
					Else If Language=1
						CurrentWorldLevelName$(i,0)="Das Tor ist geoeffnet."
						CurrentWorldLevelName$(i,1)="'Enter' zum fortsetzen."
						CurrentWorldLevelName$(i,2)=""
					Else If Language=2
						CurrentWorldLevelName$(i,0)="La Grille est ouverte."
						CurrentWorldLevelName$(i,1)="Tape 'Entree' pour continuer."
						CurrentWorldLevelName$(i,2)=""

					EndIf
				Else
					AddVertex(Surface,-1,0,1, 0,-1)
					AddVertex(Surface,1,0,1, .5,-1)
					AddVertex(Surface,1,0,-1, .5,-.5)
					AddVertex(Surface,-1,0,-1, 0,-.5)
					If Language=0
						CurrentWorldLevelName$(i,0)="The Gateway is Closed"
						CurrentWorldLevelName$(i,1)="Complete all Levels in this World"
						CurrentWorldLevelName$(i,2)="in order to open this Gate"
					Else If Language=1
						CurrentWorldLevelName$(i,0)="Das Tor ist geschlossen."
						CurrentWorldLevelName$(i,1)="Beende erst alle Levels dieser Welt"
						CurrentWorldLevelName$(i,2)="um es zu oeffnen."
					Else If Language=2
						CurrentWorldLevelName$(i,0)="La Grille est fermee."
						CurrentWorldLevelName$(i,1)="Termine tous les niveaux de ce monde"
						CurrentWorldLevelName$(i,2)="pour ouvrir cette Grille."


					EndIf

				EndIf
			Else If i=CurrentWorldLevelN+1
				; backward level
				AddVertex(Surface,-1,0,1, 0,-.5)
				AddVertex(Surface,1,0,1, .5,-.5)
				AddVertex(Surface,1,0,-1, .5,0)
				AddVertex(Surface,-1,0,-1, 0,0)
				If Language=0
					CurrentWorldLevelName$(i,0)="Gateway is Open"
					CurrentWorldLevelName$(i,1)="Press 'Enter' to Travel Back"
					CurrentWorldLevelName$(i,3)=""
				Else If Language=1
					CurrentWorldLevelName$(i,0)="Das Tor ist geoeffnet."
					CurrentWorldLevelName$(i,1)="'Enter' zum umkehren."
					CurrentWorldLevelName$(i,3)=""
				Else If Language=2
					CurrentWorldLevelName$(i,0)="La Grille est ouverte."
					CurrentWorldLevelName$(i,1)="Tape 'Entree' pour revenir en arriere."
					CurrentWorldLevelName$(i,3)=""


				EndIf
			Else 
				; normal levels
				If CurrentPlayerLevelScore(CurrentWorld*10+i)=0
					; not completed
					AddVertex(Surface,-1,0,1, 0,-1)
					AddVertex(Surface,1,0,1, .5,-1)
					AddVertex(Surface,1,0,-1, .5,-.5)
					AddVertex(Surface,-1,0,-1, 0,-.5)
				Else
					; completed
					AddVertex(Surface,-1,0,1, .5,-1)
					AddVertex(Surface,1,0,1, 1,-1)
					AddVertex(Surface,1,0,-1, 1,-.5)
					AddVertex(Surface,-1,0,-1, .5,-.5)
				EndIf
			EndIf
			AddTriangle(Surface,0,1,2)
			AddTriangle(Surface,0,2,3)
	
			UpdateNormals(CurrentWorldLevelEntity(i))
			
			RotateEntity CurrentWorldLevelEntity(i),-90,0,0

			FreeBrush Brush
						
			ScaleEntity CurrentWorldLevelEntity(i),.15,.15,.15
			
			If i<CurrentWorldLevelN
				If CurrentWorldLevelN>5
					TranslateEntity CurrentWorldLevelEntity(i),-.8+(i Mod 5)*.4,.6-Floor(i/5)*.35,0
				Else
					TranslateEntity CurrentWorldLevelEntity(i),-.8+(i Mod 5)*.4,.425,0
				EndIf
			Else If i=CurrentWorldLevelN+1
				TranslateEntity CurrentWorldLevelEntity(i),-1.3,.425,0
			Else
				TranslateEntity CurrentWorldLevelEntity(i),1.3,.425,0
			EndIf
			
			

			



			; and get the level name
			If i<CurrentWorldLevelN
				; normal world
				File=MyReadFile("levels\"+WorldLevelName$(CurrentWorld*10+i),False)
				ex$=ReadString$(File) 	
				ex$=ReadString$(File)
				ex$=ReadString$(File)
				CloseFile(File)
				If Language=1
					Restore GermanText
					; extra math is in there to chop of 5 levels if demo, so that it
					; gets the correct demo titles
					j2=0
					If Version<10 And CurrentWorld=1 Then j2=5
					For j=j2 To CurrentWorld*10+i
						Read ex$
						Read ux$
						Read ux$
						Read ux$
						Read ux$
						Read ux$
					Next
				EndIf
				If Language=2
					Restore FrenchText
					; extra math is in there to chop of 5 levels if demo, so that it
					; gets the correct demo titles
					j2=0
					If Version<10 And CurrentWorld=1 Then j2=5
					For j=j2 To CurrentWorld*10+i
						Read ex$
						Read ux$
						Read ux$
						Read ux$
						Read ux$
						Read ux$
					Next
				EndIf

							
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
				If k<2 Then CurrentWorldLevelName$(i,2)=""
				If k<1 Then CurrentWorldLevelName$(i,1)=""
			EndIf
			
								
			
		Next
		
		; do Entity #13 - the cursror
		CurrentWorldLevelEntity(12)=CreateMesh()
		Brush=CreateBrush()
		BrushTexture Brush,WorldLevelTexture2
		Surface=CreateSurface(CurrentWorldLevelEntity(12),Brush)
		AddVertex(Surface,-1,0,1, 0,0)
		AddVertex(Surface,1,0,1, 1,0)
		AddVertex(Surface,1,0,-1, 1,1)
		AddVertex(Surface,-1,0,-1, 0,1)
		AddTriangle(Surface,0,1,2)
		AddTriangle(Surface,0,2,3)
		UpdateNormals(CurrentWorldLevelEntity(12))
		FreeBrush Brush
		
		RotateEntity CurrentWorldLevelEntity(12),-90,0,0
		ScaleEntity CurrentWorldLevelEntity(12),.16,.16,.16
			
		
		UpdateWorld
		
		

	
	Else If Menu=5; In Game Menu
		; Camera - No Change
		
		; Signs
		StopSign(2)
		For i=11 To 15
			StopSign(i)
		Next
		
		; Lights
		LightColor LevelLight,0,0,0
		AmbientLight 100,100,100
		
		
		; LevelTimer - No Change
		
	Else If Menu=6; Confirm Player Delete
		; Camera
		RotateEntity Camera,0,0,0
		PositionEntity Camera,0,0,-4
		
		; Lights
		MenuBackGroundTex=MyLoadTexture("res20.dat",1,False)
		ScaleTexture MenuBackGroundTex,20,20
		EntityTexture MenuBackGround,MenuBackGroundTex
		MenuBackGroundTex2=MyLoadTexture("res21.dat",1,False)
		ScaleTexture MenuBackGroundTex2,20,20
		EntityTexture MenuBackGround2,MenuBackGroundTex2
		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255	


		; Music
		;If ChannelPlaying(Music)=True
		;	StopChannel(Music)
		;EndIf
		
		; BackGround
		ShowEntity MenuBackGround
		ShowEntity MenuBackGround2
		ShowEntity MenuSelector
		HideEntity Logo1
		HideEntity Logo2
	Else If Menu=7; The PlayGround PreAmble
		; Camera
		RotateEntity Camera,0,0,0
		PositionEntity Camera,0,0,-4
		
		; Lights
		MenuBackGroundTex=MyLoadTexture("res20.dat",1,False)
		ScaleTexture MenuBackGroundTex,20,20
		EntityTexture MenuBackGround,MenuBackGroundTex
		MenuBackGroundTex2=MyLoadTexture("res21.dat",1,False)
		ScaleTexture MenuBackGroundTex2,20,20
		EntityTexture MenuBackGround2,MenuBackGroundTex2
		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255	


		; Music
		;If ChannelPlaying(Music)=True
		;	StopChannel(Music)
		;EndIf
		
		; BackGround
		ShowEntity MenuBackGround
		ShowEntity MenuBackGround2
		ShowEntity MenuSelector
		HideEntity Logo1
		HideEntity Logo2
		
	Else If Menu=8; PlayGround Level Selector
		; Camera
		RotateEntity Camera,0,0,0
		PositionEntity Camera,0,0,-4
		
		; Lights
		MenuBackGroundTex=MyLoadTexture("res22.dat",1,False)
		ScaleTexture MenuBackGroundTex,20,20
		EntityTexture MenuBackGround,MenuBackGroundTex
		EntityTexture MenuBackGround2,MenuBackGroundTex

		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255
		
		; Music
		If CurrentMenu<>8 And DoesSoundExist=True
			StartLevelMusic()
		EndIf

		
		; BackGround
		ShowEntity MenuBackGround
		ShowEntity MenuBackGround2
		HideEntity MenuSelector
		HideEntity Logo1
		HideEntity Logo2

		LevelTimer=0
		
		; Special Stuff
		
		; add the squares for the levels
		; how many levels?
		CurrentWorldLevelN=10
		CurrentLevel=PlayGroundLevel
		CurrentWorld=PlayGroundLevel/10
				
		
		; create the entities
		NextWorldFlag=0
		For i=0 To 9
		
			; create the flat squares for each level
			
			CurrentWorldLevelEntity(i)=CreateMesh()
			Brush=CreateBrush()
			BrushTexture Brush,WorldLevelTexture
			Surface=CreateSurface(CurrentWorldLevelEntity(i),Brush)
					
			; completed
			AddVertex(Surface,-1,0,1, .5,-1)
			AddVertex(Surface,1,0,1, 1,-1)
			AddVertex(Surface,1,0,-1, 1,-.5)
			AddVertex(Surface,-1,0,-1, .5,-.5)
					
			AddTriangle(Surface,0,1,2)
			AddTriangle(Surface,0,2,3)
	
			UpdateNormals(CurrentWorldLevelEntity(i))
			
			RotateEntity CurrentWorldLevelEntity(i),-90,0,0

			FreeBrush Brush
						
			ScaleEntity CurrentWorldLevelEntity(i),.15,.15,.15
			
			If i<CurrentWorldLevelN
				If CurrentWorldLevelN>5
					TranslateEntity CurrentWorldLevelEntity(i),-.8+(i Mod 5)*.4,.6-Floor(i/5)*.35,0
				Else
					TranslateEntity CurrentWorldLevelEntity(i),-.8+(i Mod 5)*.4,.425,0
				EndIf
			EndIf
			
			

			



			; and get the level name
			If i<CurrentWorldLevelN
				; normal world
				File=MyReadFile("levels\"+WorldLevelName$(PlayGroundLevel+i),False)
				ex$=ReadString$(File) 	
				ex$=ReadString$(File)
				ex$=ReadString$(File)
				CloseFile(File)
				If Language=1
					Restore GermanText
					For j=0 To PlayGroundLevel+i
						If j<100 Or j>399
							Read ex$
							Read ux$
							Read ux$
							Read ux$
							Read ux$
							Read ux$
						EndIf
					Next
				EndIf
				If Language=2
					Restore FrenchText
					For j=0 To PlayGroundLevel+i
						If j<100 Or j>399
							Read ex$
							Read ux$
							Read ux$
							Read ux$
							Read ux$
							Read ux$
						EndIf
					Next
				EndIf


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
				If k<2 Then CurrentWorldLevelName$(i,2)=""
				If k<1 Then CurrentWorldLevelName$(i,1)=""
			EndIf
			
								
			
		Next
		
		; do Entity #13 - the cursror
		CurrentWorldLevelEntity(12)=CreateMesh()
		Brush=CreateBrush()
		BrushTexture Brush,WorldLevelTexture2
		Surface=CreateSurface(CurrentWorldLevelEntity(12),Brush)
		AddVertex(Surface,-1,0,1, 0,0)
		AddVertex(Surface,1,0,1, 1,0)
		AddVertex(Surface,1,0,-1, 1,1)
		AddVertex(Surface,-1,0,-1, 0,1)
		AddTriangle(Surface,0,1,2)
		AddTriangle(Surface,0,2,3)
		UpdateNormals(CurrentWorldLevelEntity(12))
		FreeBrush Brush
		
		RotateEntity CurrentWorldLevelEntity(12),-90,0,0
		ScaleEntity CurrentWorldLevelEntity(12),.16,.16,.16
			
		
		UpdateWorld
		
	Else If Menu=9; Options Menu
		; Camera
		RotateEntity Camera,0,0,0
		PositionEntity Camera,0,0,-4
		
		; Lights
		MenuBackGroundTex=MyLoadTexture("res20.dat",1,False)
		ScaleTexture MenuBackGroundTex,20,20
		EntityTexture MenuBackGround,MenuBackGroundTex
		MenuBackGroundTex2=MyLoadTexture("res21.dat",1,False)
		ScaleTexture MenuBackGroundTex2,20,20
		EntityTexture MenuBackGround2,MenuBackGroundTex2
		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255	


		; Music
		;If ChannelPlaying(Music)=True
		;	StopChannel(Music)
		;EndIf
		
		; BackGround
		ShowEntity MenuBackGround
		ShowEntity MenuBackGround2
		ShowEntity MenuSelector
		HideEntity Logo1
		HideEntity Logo2

		
	Else If Menu=10; exit to windows?
		; Camera
		RotateEntity Camera,0,0,0
		PositionEntity Camera,0,0,-4
		
		; Lights
		MenuBackGroundTex=MyLoadTexture("res20.dat",1,False)
		ScaleTexture MenuBackGroundTex,20,20
		EntityTexture MenuBackGround,MenuBackGroundTex
		MenuBackGroundTex2=MyLoadTexture("res21.dat",1,False)
		ScaleTexture MenuBackGroundTex2,20,20
		EntityTexture MenuBackGround2,MenuBackGroundTex2
		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255	


		; Music
		;If ChannelPlaying(Music)=True
		;	StopChannel(Music)
		;EndIf
		
		; BackGround
		ShowEntity MenuBackGround
		ShowEntity MenuBackGround2
		ShowEntity MenuSelector
		HideEntity Logo1
		HideEntity Logo2
		
	Else If Menu=11; shareware demo
		; Camera
		RotateEntity Camera,0,0,0
		PositionEntity Camera,0,0,-4
		
		; Lights
		MenuBackGroundTex=MyLoadTexture("res20.dat",1,False)
		ScaleTexture MenuBackGroundTex,20,20
		EntityTexture MenuBackGround,MenuBackGroundTex
		MenuBackGroundTex2=MyLoadTexture("res21.dat",1,False)
		ScaleTexture MenuBackGroundTex2,20,20
		EntityTexture MenuBackGround2,MenuBackGroundTex2
		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255	


		; Music
		If DoesSoundExist=True And CurrentMenu<>7 And CurrentMenu<>9 And CurrentMenu<>10
			StopChannel(Music)
			Music=PlayMusic("res4.dat")
			ChannelVolume music,OptionsMusicLevel
		EndIf

		
		; BackGround
		ShowEntity MenuBackGround
		ShowEntity MenuBackGround2
		ShowEntity MenuSelector
		HideEntity Logo1
		HideEntity Logo2
		
		TitleChar(0)=CopyEntity(StinkyMesh)
		EntityTexture TitleChar(0),StinkyTexture
		RotateEntity TitleChar(0),45,190,0
		;ScaleEntity TitleChar(0),.05,.05,.05
		PositionEntity TitleChar(0),-5,-5,0
		AnimateMD2 TitleChar(0),1,0.15,44,52
		EntityAlpha TitleChar(0),1

		
		; and demopics
		TitleChar(3)=CreateCube()
		ScaleEntity TitleChar(3),1,1,0.001
		EntityTexture TitleChar(3),DemoTexture(0)
	;	ScaleSprite TitleChar(0),.0,.0
		PositionEntity TitleChar(3),.7,.5,0
		PositionEntity TitleChar(3),3,.5,0

		
		TitleChar(1)=CreateCube()
		ScaleEntity TitleChar(1),1,1,0.001
		EntityTexture TitleChar(1),DemoTexture(1)
	;	ScaleSprite TitleChar(1),.0,.0
		PositionEntity TitleChar(1),3,.5,0


		TitleChar(2)=CreateCube()
		ScaleEntity TitleChar(2),1,1,0.001
		EntityTexture TitleChar(2),DemoTexture(2)
	;	ScaleSprite TitleChar(2),.0,.0
		PositionEntity TitleChar(2),3,.5,0




		
		
		LevelTimer=0
		
	Else If Menu=12	; End of Game
		
		
		; Lights
		MenuBackGroundTex=MyLoadTexture("res21.dat",1,False)
		ScaleTexture MenuBackGroundTex,20,20
		EntityTexture MenuBackGround,MenuBackGroundTex
		MenuBackGroundTex2=MyLoadTexture("res20.dat",1,False)
		ScaleTexture MenuBackGroundTex2,20,20
		EntityTexture MenuBackGround2,MenuBackGroundTex2
		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255	

		
		
		; BackGround
		;ShowEntity MenuBackGround
		;ShowEntity MenuBackGround2
		;
		;ShowEntity MenuSelector
		;ShowEntity Logo1
		;ShowEntity Logo2
		
	
		
		
		StartLevel(PlayGroundLevel+10)

		TranslateEntity StinkyEntity,0.2,.9,0.05
		TranslateEntity LoofEntity,0,.9,0
		TurnEntity StinkyEntity,0,160,0
		TurnEntity LoofEntity,0,170,0
		EntityTexture StinkyEntity,StinkyTextureC
		EntityTexture LoofEntity,LoofTextureC

		; Camera
		RotateEntity Camera,0,0,0
		PositionEntity Camera,25,0,-100
		
		CameraClsColor Camera,170,170,255
		CameraFogColor Camera,170,170,255
		CameraRange Camera,.98,1000
		CameraFogRange Camera,100,1000
		CameraFogMode Camera,1

		
		ScaleTexture BackGroundTexture,600,600
		ShiftKeyPressed=False
		CursorKeyPressed=True

			
		LevelTimer=0
		
		; Music
		If DoesSoundExist=True And CurrentMenu<>7 And CurrentMenu<>9 And CurrentMenu<>10
			StopChannel(Music)
			Music=PlayMusic("res7.dat")
			ChannelVolume music,OptionsMusicLevel
		EndIf

;LightColor LevelLight,255,255,255
;		AmbientLight 255,255,255	
		NofFireballs=50
		For i=0 To 50
			FireBallEntity(i)=CreateCube()
			ScaleEntity FireBallEntity(i),1,1,0.001
			EntityTexture FireBallEntity(i),ParticleTexture,3
			ScaleEntity FireBallEntity(i),.2,.2,0.001
		Next 



	EndIf
	
	CurrentMenu=Menu
	CurrentMenuOption=0
	GamePhase=0
		
	TweenPeriod=1000/85
	TweenTime=MilliSecs()-TweenPeriod


End Function

Function MenuLoop()
	
	If CurrentMenu<>5 Then LevelTimer=LevelTimer+1

	If CurrentMenu=0 Then MenuLoop0
	If CurrentMenu=1 Then MenuLoop1
	If CurrentMenu=2 Then MenuLoop2
	If CurrentMenu=3 Then MenuLoop3
	If CurrentMenu=4 Then MenuLoop4
	If CurrentMenu=5 Then MenuLoop5
	If CurrentMenu=6 Then MenuLoop6
	If CurrentMenu=7 Then MenuLoop7
	If CurrentMenu=8 Then MenuLoop8
	If CurrentMenu=9 Then MenuLoop9
	If CurrentMenu=10 Then MenuLoop10
	If CurrentMenu=11 
		If Language=0 
			MenuLoop11
		Else
			MenuLoop11old
		EndIf
	EndIf
	If CurrentMenu=12 Then MenuLoop12


	If KeyHit(66) Then
		SaveBuffer(FrontBuffer(),"screenshot.bmp")
	End If


	
End Function

Function MenuLoop0()

	
		; Display Static Text
	
	i=0
	
	If (Version=11)
		CenterText("Welcome to Wonderland",2+i,1,50)
		CenterText("-------------------------",3+i,1,51)
		CenterText("The following is a complimentary",5+i,1,52)
		CenterText("review copy of Wonderland.",6+i,1,53)
		CenterText("This copy is for evaluation purposes only",8+i,1,54)
		CenterText("and is not to be distributed.",9+i,1,55)
		CenterText("For further information please visit",10+i,1,56)
		CenterText("WWW.MIDNIGHTSYNERGY.COM",11+i,1,57)
		CenterText("Thanks for playing!",13+i,1,58)
		
		
		CenterText("PRESS ENTER TO START",15,1,63)
	Else
		If Language=0
			CenterText("Welcome to Wonderland",2+i,1,50)
			CenterText("-------------------------",3+i,1,51)
			CenterText("Thank you for trying out the",5+i,1,52)
			CenterText("free demo version of Wonderland.",6+i,1,53)
			CenterText("If you enjoy this game, please consider",8+i,1,54)
			CenterText("supporting independent shareware games",9+i,1,55)
			CenterText("by ordering the full version at",10+i,1,56)
			CenterText("WWW.MIDNIGHTSYNERGY.COM",11+i,1,57)
			CenterText("The full version includes 85 (!) levels,",13+i,1,58)
			CenterText("ten worlds, as well as new features,",14+i,1,59)
			CenterText("new graphics, and new music.",15+i,1,60)
			
			
			CenterText("PRESS ENTER TO START",17,1,63)
		Else If Language=1
			CenterText("Willkommen zu Wonderland",2+i,1,50)
			CenterText("-------------------------",3+i,1,51)
			CenterText("Dies ist die Freiversion",5+i,1,52)
			CenterText("von Wonderland.",6+i,1,53)
			CenterText("Wenn Ihnen diese Demo gefaellt,",8+i,1,54)
			CenterText("koennen Sie sich die Vollversion",9+i,1,55)
			CenterText("bei WWW.MIDNIGHTSYNERGY.COM",10+i,1,56)
			CenterText("bestellen.",11+i,1,57)
			CenterText("Die Vollversion hat 85 Levels (!),",13+i,1,58)
			CenterText("zehn Welten, mehr Graphiken,",14+i,1,59)
			CenterText("mehr Sound, und mehr Musik.",15+i,1,60)
			CenterText("'ENTER'",17,1,63)
		Else If Language=2
			CenterText("Bienvenue a Wonderland",2+i,1,50)
			CenterText("----------------------",3+i,1,51)
			CenterText("Merci d'essayer la demo",5+i,1,52)
			CenterText("gratuit du jeu 'Wonderland'.",6+i,1,53)
			CenterText("Si vous aimez ce jeu, s.v.p. songez a",8+i,1,54)
			CenterText("encourager les jeux partagiciels independants",9+i,1,55)
			CenterText("en vous procurant la version complete de",10+i,1,56)
			CenterText("WWW.MIDNIGHTSYNERGY.COM",11+i,1,57)
			CenterText("Il y a 85 (!) niveaux dans la version complete,",13+i,1,58)
			CenterText("dix mondes, avec de nouvelles fonctions,",14+i,1,59)
			CenterText("de nouveaux designs, et de nouvelles musiques.",15+i,1,60)
			CenterText("Tape ENTREE POUR COMMENCER",17,1,63)


		EndIf
		i=2
	EndIf
	 
	
	
		
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(15+i)*.044+.027,1.1
	ScaleEntity MenuSelector,23.5*.012,.02,0.001
	

	
	
	
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
		PositionTexture MenuBackGroundTex,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		PositionTexture MenuBackGroundTex2,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
	
	
		
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	
	
	If (KeyDown(28) Or KeyDown(156))
		If CursorKeyPressed=False
			CursorKeyPressed=True
			EndMenu()
			StartMenu(1)
		EndIf
	
	Else
		CursorKeyPressed=False
	EndIf


	

End Function

Function MenuLoop1()
	
	
	; Display Static Text
	
	DisplayText("v"+Str$(VersionNo1)+"."+Str$(VersionNo2)+Str$(VersionNo3),44,0,1,50)
	If Version<10 And LevelTimer Mod 200 < 100
		DisplayText("DEMO",46,1,1,62)
	Else
		DisplayText("",46,1,1,62)
	EndIf
	If Language=0
		CenterText("START GAME",11,1,52)
		CenterText("THE PLAYGROUND",12,1,53)
		CenterText("OPTIONS",13,1,54)
		CenterText("ONLINE SOLUTIONS",14,1,63)
		CenterText("EXIT TO WINDOWS",15,1,55)
		If Version<10 CenterText("GET THE FULL VERSION",16,1,56)
	Else If Language=1
		CenterText("SPIEL STARTEN",11,1,52)
		CenterText("BEGINNER LEVELS",12,1,53)
		CenterText("OPTIONEN",13,1,54)
		CenterText("ONLINE HILFE",14,1,63)
		CenterText("SPIEL BEENDEN",15,1,55)
		If Version<10 CenterText("VOLLVERSION KAUFEN",16,1,56)
	Else If Language=2
		CenterText("JOUER",11,1,52)
		CenterText("LE TERRAIN DE JEU",12,1,53)
		CenterText("OPTIONS",13,1,54)
		CenterText("SOLUCES EN LIGNE",14,1,63)
		CenterText("SORTIE VERS WINDOWS",15,1,55)
		If Version<10 CenterText("VERSION COMPLETE",16,1,56)

	EndIf
	
	
;	CenterText("TRY OTHER GAMES",16,1,70) ; Gamextazy

	If LevelTimer Mod 200 <100
		If Language=0
			CenterText("Use Cursor Keys and Enter to Select",17,1,57)
		Else If Language=1
			CenterText("Benutze die Pfeiltasten und zur Auswahl 'Enter'",17,1,57)
		Else If Language=2
			CenterText("Pour choisir, utilise les fleches et 'Entree'",17,1,57)
		EndIf

	Else 
		CenterText("",17,1,57)
	EndIf
	; ScrollText
	LevelTimer2=(LevelTimer Mod (Len(ScrollText$)*50))
	
	ScrollText2$=Mid$(ScrollText$,LevelTimer2/50+1,51)
		
	DisplayText(ScrollText2$,-Float(LevelTimer Mod 50)/50.0,18,1,51)
	
;	PROMOSHOT	
;	LevelTimer=0
;	AnimateMD2 TitleChar(0),0,0,0,0
;	ScaleEntity TitleChar(0),1,1,1
;	PositionEntity TitleChar(0),-.9,-1,.6
;	RotateEntity TitleChar(0),25,210,-5
;	ShowEntity TitleChar(0)
;	EntityAlpha TitleChar(0),1
;	ScaleEntity TitleChar(1),.7,.7,.7
;	PositionEntity TitleChar(1),0,-.9,.7
;	RotateEntity TitleChar(1),25,210,-5
;	ShowEntity TitleChar(1)
;	EntityAlpha TitleChar(1),1


	

	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(11+CurrentMenuOption)*.044+.027,1.1

	If CurrentMenuOption=0	
		ScaleEntity MenuSelector,12.5*.012,.02,0.001
	Else If CurrentMenuOption=1
		ScaleEntity MenuSelector,18.5*.012,.02,0.001
	Else If CurrentMenuOption=2
		ScaleEntity MenuSelector,7.5*.012,.02,0.001
	Else If CurrentMenuOption=3
		ScaleEntity MenuSelector,18.5*.012,.02,0.001

	Else If CurrentMenuOption=4
		ScaleEntity MenuSelector,18.5*.012,.02,0.001
	Else If CurrentMenuOption=5
		ScaleEntity MenuSelector,20.5*.012,.02,0.001
	EndIf

	
	
	LevelTimer2=LevelTimer Mod 14000
	
	RotateEntity TitleChar(2),-90,90,LevelTimer2 Mod 360
	
	k#=Float(LevelTimer2 Mod 100)/50.0
	PositionEntity TitleChar(3),-.4,.4+.4*k*(2-k),0
	ScaleEntity TitleChar(3),.2,.1+.1*k*(2-k),.2

	
	PositionEntity TitleChar(5),1.2+.05*Cos((LevelTimer2 Mod 360)*3),.45+.1*Sin(LevelTimer2 Mod 360),0
	RotateEntity TitleChar(5),-90,10*Cos((LevelTimer2 Mod 360)),90

	
	
	
	If LevelTimer2<1000
		CenterText("",0,1,58)
		CenterText("",0,1,59)
		CenterText("",0,1,60)

		; Wonderland Zoom
		PositionEntity Logo1,-2.99*Float(LevelTimer2)/1000,.6+Float(LevelTimer2)/700,0
		PositionEntity Logo2,0,.6+Float(LevelTimer2)/700,0
		ScaleEntity Logo1,3*Float(LevelTimer2)/1000.0,3*Float(LevelTimer2)/1000.0,3*Float(LevelTimer2)/1000.0
		ScaleEntity Logo2,3*Float(LevelTimer2)/1000.0,3*Float(LevelTimer2)/1000.0,3*Float(LevelTimer2)/1000.0
		If LevelTimer2<500
			EntityAlpha Logo1,Float(LevelTimer2)/600.0
			EntityAlpha Logo2,Float(LevelTimer2)/600.0
		Else
			EntityAlpha Logo1,(1000-Float(LevelTimer2))/600.0
			EntityAlpha Logo2,(1000-Float(LevelTimer2))/600.0
		EndIf
	Else If LevelTimer2<1250
		; Pause
		CenterText("WONDERLAND",4,1,59)
		EntityAlpha Logo1,0
		EntityAlpha Logo2,0

	Else If LevelTimer2<1500
		; Pause
		CenterText("WONDERLAND",4,1,59)
		If Language=0
			CenterText("A MIDNIGHT SYNERGY GAME",6,1,58)
		Else If Language=1
			CenterText("EIN MIDNIGHT SYNERGY SPIEL",6,1,58)
		Else If Language=2
			CenterText("UN JEU MIDNIGHT SYNERGY",6,1,58)
		EndIf
		EntityAlpha Logo1,0
		EntityAlpha Logo2,0
	Else If LevelTimer2<1600
		CenterText("",0,1,58)
		CenterText("",0,1,59)
	Else If LevelTimer2<2000
		If Language=0
			CenterText("STARRING",5,1,58)
		Else If Language=1
			CenterText("MIT",5,1,58)
		Else If Language=2
			CenterText("EN VEDETTE:",5,1,58)
		EndIf
	Else If LevelTimer2<2400
		; Stinky
		EntityAlpha TitleChar(0),1
		DisplayText("STINKY",11,2,1,58)
		If LevelTimer2<2200
			RotateEntity TitleChar(0),45,191,0
			ScaleEntity TitleChar(0),.5*(LevelTimer2-2000)/200,.5*(LevelTimer2-2000)/200,.5*(LevelTimer2-2000)/200
			PositionEntity TitleChar(0),-.86,.6-.2*(LevelTimer2-2000)/200,0
		EndIf
	Else If LevelTimer2<2600
		; Stinky and
		DisplayText("STINKY",11,2,1,58)
		If Language=0
			CenterText("AND",5,1,59)
		Else If Language=1
			CenterText("UND",5,1,59)
		Else If Language=2
			CenterText("ET",5,1,59)

		EndIf
	Else If LevelTimer2<3200
		; Stinky and Loof
		DisplayText("STINKY",11,2,1,58)
		If Language=0
			CenterText("AND",5,1,59)
		Else If Language=1
			CenterText("UND",5,1,59)
		Else If Language=2
			CenterText("ET",5,1,59)

		EndIf
		DisplayText("LOOF",34,8,1,60)
		EntityAlpha TitleChar(1),1
		If LevelTimer2<2900
			ScaleEntity TitleChar(1),.4*(LevelTimer2-2600)/300,.4*(LevelTimer2-2600)/300,.4*(LevelTimer2-2600)/300
			PositionEntity TitleChar(1),.86,.6-.2*(LevelTimer2-2600)/300,0
			RotateEntity TitleChar(1),45,170,0
			TurnEntity TitleChar(1),0,(LevelTimer2-2600)*720.0/150.0,0
		Else 
			RotateEntity TitleChar(1),45,170,0
		EndIf
	Else If LevelTimer2<3400
		; Spin Away Stinky and Loof
		DisplayText("",0,0,1,58)
		DisplayText("",0,0,1,59)
		DisplayText("",0,0,1,60)
		TurnEntity TitleChar(0),0,720.0/50.0,0
		TurnEntity TitleChar(1),0,720.0/50.0,0
		TranslateEntity TitleChar(0),0,.025,0
		TranslateEntity TitleChar(1),0,.025,0
		ScaleEntity TitleChar(0),.4*(3400-LevelTimer2)/200,.4*(3400-LevelTimer2)/200,.4*(3400-LevelTimer2)/200
		ScaleEntity TitleChar(1),.4*(3400-LevelTimer2)/200,.4*(3400-LevelTimer2)/200,.4*(3400-LevelTimer2)/200
	Else If LevelTimer2<4000
		EntityAlpha TitleChar(0),0
		EntityAlpha TitleChar(1),0
		; Also Featuring
		If Language=0
			DisplayText("ALSO FEATURING",50-0.12*(Leveltimer2-3400),5-3*Abs(Cos((LevelTimer2-3400)*2)),1,58)
		Else If Language=1
			DisplayText("    UND...    ",50-0.12*(Leveltimer2-3400),5-3*Abs(Cos((LevelTimer2-3400)*2)),1,58)
		Else If Language=2
			DisplayText("  AVEC AUSSI  ",50-0.12*(Leveltimer2-3400),5-3*Abs(Cos((LevelTimer2-3400)*2)),1,58)
		EndIf
	
	Else If LevelTimer2<4400
		RotateEntity TitleChar(2),-90,90,LevelTimer2 Mod 360
		EntityAlpha TitleChar(2),1
		ScaleEntity TitleChar(2),.45*(LevelTimer2-4000)/400,.45*(LevelTimer2-4000)/400,.45*(LevelTimer2-4000)/400
		DisplayText("SCOUGE",7,7.8,1,58)
	Else If LevelTimer2<4800
		RotateEntity TitleChar(2),-90,90,LevelTimer2 Mod 360
		k#=Float(LevelTimer2 Mod 100)/50.0
		PositionEntity TitleChar(3),-.4,.4+.4*k*(2-k),0
		ScaleEntity TitleChar(3),.2,.1+.1*k*(2-k),.2

		EntityAlpha TitleChar(3),1
		ScaleEntity TitleChar(3),.2*(LevelTimer2-4400)/400,.2*(LevelTimer2-4400)/400,.2*(LevelTimer2-4400)/400
		DisplayText("SCOUGE     COILY",7,7.8,1,58)
	Else If LevelTimer2<5200
		RotateEntity TitleChar(2),-90,90,LevelTimer2 Mod 360
	
		k#=Float(LevelTimer2 Mod 100)/50.0
		PositionEntity TitleChar(3),-.4,.4+.4*k*(2-k),0
		ScaleEntity TitleChar(3),.2,.1+.1*k*(2-k),.2
	
		EntityAlpha TitleChar(4),1
		ScaleEntity TitleChar(4),.45*(LevelTimer2-4800)/400,.45*(LevelTimer2-4800)/400,.45*(LevelTimer2-4800)/400
		DisplayText("SCOUGE     COILY    KABOOM!",7,7.8,1,58)
	Else If LevelTimer2<5600
		RotateEntity TitleChar(2),-90,90,LevelTimer2 Mod 360
	
		k#=Float(LevelTimer2 Mod 100)/50.0
		PositionEntity TitleChar(3),-.4,.4+.4*k*(2-k),0
		ScaleEntity TitleChar(3),.2,.1+.1*k*(2-k),.2
	
		
		PositionEntity TitleChar(5),1.2+.05*Cos((LevelTimer2 Mod 360)*3),.45+.1*Sin(LevelTimer2 Mod 360),0
		RotateEntity TitleChar(5),-90,10*Cos((LevelTimer2 Mod 360)),90

		EntityAlpha TitleChar(5),1
		ScaleEntity TitleChar(5),.25*(LevelTimer2-5200)/400,.25*(LevelTimer2-5200)/400,.25*(LevelTimer2-5200)/400
		If Language=0
			DisplayText("SCOUGE     COILY    KABOOM!  and Z-BOT",7,7.8,1,58)
		Else If Language=1
			DisplayText("SCOUGE     COILY    KABOOM!  und Z-BOT",7,7.8,1,58)
		Else If Language=2
			DisplayText("SCOUGE     COILY    KABOOM!  et ZI-BOT",7,7.8,1,58)
		EndIf
	Else If LevelTimer2<5800
	Else If LevelTimer2<6398
		DisplayText("",1,1,1,58)
		
		PositionEntity TitleChar(2),EntityX(TitleChar(2)),EntityY(TitleChar(2)),.13*Float(LevelTimer2-5800)
		PositionEntity TitleChar(3),EntityX(TitleChar(3)),EntityY(TitleChar(3)),.13*Float(LevelTimer2-5800)
		PositionEntity TitleChar(4),EntityX(TitleChar(4)),EntityY(TitleChar(4)),.13*Float(LevelTimer2-5800)
		PositionEntity TitleChar(5),EntityX(TitleChar(5)),EntityY(TitleChar(5)),.13*Float(LevelTimer2-5800)
		
	Else If LevelTimer2<7000
		PositionEntity TitleChar(2),-1.2,.4,0
		PositionEntity TitleChar(3),-.4,.4,0
		PositionEntity TitleChar(4),.4,.4,0
		PositionEntity TitleChar(5),1.2,.4,0

		For k=2 To 5
			EntityAlpha TitleChar(k),0
			ScaleEntity TitleChar(k),0.01,0.01,0.01
		Next
	
	Else If LevelTimer2<8000
		; Wonderland Zoom
		PositionEntity Logo1,-2.99*Float(LevelTimer2-7000)/1000,.6+Float(LevelTimer2-7000)/700,0
		PositionEntity Logo2,0,.6+Float(LevelTimer2-7000)/700,0
		ScaleEntity Logo1,3*Float(LevelTimer2-7000)/1000.0,3*Float(LevelTimer2-7000)/1000.0,3*Float(LevelTimer2-7000)/1000.0
		ScaleEntity Logo2,3*Float(LevelTimer2-7000)/1000.0,3*Float(LevelTimer2-7000)/1000.0,3*Float(LevelTimer2-7000)/1000.0
		If LevelTimer2<7500
			EntityAlpha Logo1,Float(LevelTimer2-7000)/600.0
			EntityAlpha Logo2,Float(LevelTimer2-7000)/600.0
		Else
			EntityAlpha Logo1,(8000-Float(LevelTimer2))/600.0
			EntityAlpha Logo2,(8000-Float(LevelTimer2))/600.0
		EndIf
		
	Else If LevelTimer2<10000
		EntityAlpha Logo1,0
		EntityAlpha Logo2,0
	Else If LevelTimer2<11000
		; Wonderland Zoom
		PositionEntity Logo1,-2.99*Float(LevelTimer2-10000)/1000,.6+Float(LevelTimer2-10000)/700,0
		PositionEntity Logo2,0,.6+Float(LevelTimer2-10000)/700,0
		ScaleEntity Logo1,3*Float(LevelTimer2-10000)/1000.0,3*Float(LevelTimer2-10000)/1000.0,3*Float(LevelTimer2-10000)/1000.0
		ScaleEntity Logo2,3*Float(LevelTimer2-10000)/1000.0,3*Float(LevelTimer2-10000)/1000.0,3*Float(LevelTimer2-10000)/1000.0
		If LevelTimer2<10500
			EntityAlpha Logo1,Float(LevelTimer2-10000)/600.0
			EntityAlpha Logo2,Float(LevelTimer2-10000)/600.0
		Else
			EntityAlpha Logo1,(11000-Float(LevelTimer2))/600.0
			EntityAlpha Logo2,(11000-Float(LevelTimer2))/600.0
		EndIf
		
	Else
		EntityAlpha Logo1,0
		EntityAlpha Logo2,0

	EndIf
	
	
	
	
	; turn on text and signs
	For l.LetterType=Each LetterType
		If l\Active=False
			l\Active=True
			ShowEntity l\Entity
		EndIf
	Next


	
	
	
	


	
	;If LevelTimer<64
	;	LightColor LevelLight,-255+LevelTimer*8,-255+LevelTimer*8,-255+LevelTimer*8
	;	AmbientLight LevelTimer*4,LevelTimer*4,LevelTimer*4
	;EndIf

	
	
	

	Repeat
		TweenElapsed=MilliSecs()-TweenTime
	Until TweenElapsed>TweenPeriod
	

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
		
	
		
		
		
		PositionTexture MenuBackGroundTex,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		PositionTexture MenuBackGroundTex2,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0

	
		
	
	;	TurnEntity Logo,0.4*Cos((LevelTimer/2) Mod 360),0,0
		
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)		
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=5-Floor(Version/10)
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption+1
			If CurrentMenuOption=6-Floor(Version/10) Then CurrentMenuOption=0
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
				StartMenu(7)
				Return
			EndIf
			If CurrentMenuOption=2
				EndMenu()
				StartMenu(9)
				Return
			EndIf
			If CurrentMenuOption=3
				EndMenu()
				VersionNo1=-999
				GameExit=True
			EndIf
			If CurrentMenuOption=4
				EndMenu()
				StartMenu(10)
			EndIf
			If CurrentMenuOption=5
				EndMenu()
				; go to site!
				VersionNo1=-1
				GameExit=True
			EndIf
			
		;	If currentmenuoption=5
		;		If DoesSoundExist=True
		
		;			For i=1 To 100
		;				ChannelVolume Music,1-Float(i)/100.0
		;				Delay 10
		;			Next 
		;			StopChannel(Music)
	
		;		EndIf
;
;				ExecFile("http://www.gamextazy.com/?from_game_Wonderland")
;				End
;			EndIf
				
			
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
	
	If Version>=10
		If KeyDown(2) And KeyDown(3) And KeyDown(4) And Version=11
			EndMenu()
			StartMenu(12)
		EndIf
		If (KeyDown(29) Or KeyDown(157)) And KeyDown(62) And Version<>11
			EndMenu()
			StartMenu(12)
		EndIf
		
	EndIf
	
	; Cheat 
	If Version<10
		If (KeyDown(29) Or KeyDown(157)) And (KeyDown(42) Or KeyDown(54)) And KeyDown(10) And IsExtraDemoLevelActive=0
			PlayMySound(SFXLastCoin,0)
			IsExtraDemoLevelActive=1
			SaveOptions()
			WorldLevelName$(15)="resdb1.dat"
			WorldLevelName$(16)="resdb2.dat"
		EndIf
	EndIf

		
		
	

End Function

Function MenuLoop2()
	

	; Display Static Text
	If Language=0
		CenterText("SELECT PLAYER",4,1,50)
		CenterText("------------------------------",5,1,51)
		DisplayText("Name             Levels  Score",10,6,1,52)
		
		DisplayText("CREATE NEW PLAYER",10,7,1,53)
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
			DisplayText(Upper$(PlayerName$(i)),10,8+i,1,54+3*i)
			DisplayText(PlayerLevels(i),31-Len(Str$(PlayerLevels(i))),8+i,1,55+3*i)
			DisplayText(PlayerScore(i),40-Len(Str$(PlayerScore(i))),8+i,1,56+3*i)
		Next 
	EndIf
	CenterText("------------------------------",8+PlayerNameN,1,54+3*PlayerNameN)

	
		
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(7+CurrentMenuOption)*.044+.027,1.1
	ScaleEntity MenuSelector,30.5*.012,.02,0.001
	

	
	
	
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
		PositionTexture MenuBackGroundTex,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		PositionTexture MenuBackGroundTex2,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
	
		
		
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
		PositionTexture MenuBackGroundTex,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		PositionTexture MenuBackGroundTex2,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0

		
		
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	
	; keyboard needs to be put in here
;	If KeyDown(16) And Len(CurrentPlayerName$)<15 
;		If CursorKeyPressd=False
;			CurrentPlayerName$=CurrentPlayerName$+"Q"
;			CursorKeyPressed=True
;		EndIf
;		
;	Else If KeyDown(17) And Len(CurrentPlayerName$)<15 
;		If CursorKeyPressd=False
;			CurrentPlayerName$=CurrentPlayerName$+"W"
;			CursorKeyPressed=True
;		EndIf
	
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
			StartMenu(4)
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

Function MenuLoop4()

	; *******************
	
	; FIRST: load up the world data, i.e. each levelname, score, world name
	; *******************
	DisplayText(CurrentPlayerName$,0,0,1,61)
	AstragonLess=0
	If Version=12 And CurrentWorld>7
		AstragonLess=2
	EndIf
	If Language=0
		CenterText("WORLD "+Str$(CurrentWorld+1-AstragonLess),2,1,50)
	Else If Language=1
		CenterText("WELT  "+Str$(CurrentWorld+1-AstragonLess),2,1,50)
	Else If Language=2
		CenterText("MONDE  "+Str$(CurrentWorld+1-AstragonLess),2,1,50)

	EndIf
	CenterText(WorldName$(CurrentWorld,Language),3,1,51)
	For i=0 To PlayGroundLevel-1
		If CurrentPlayerLevelScore(i)>0
			j=j+CurrentPlayerLevelScore(i)
		EndIf
	Next
	If Language=0
		DisplayText("TOTAL SCORE:",38,0,1,52)
	Else If Language=1
		DisplayText("     PUNKTE:",38,0,1,52)
	Else If Language=2
		DisplayText("SCORE TOTAL:",38,0,1,52)

	EndIf
	DisplayText(Str$(j),50-Len(Str$(j)),1,1,53)	
	
			
	
	For i=0 To 2
		CenterText(CurrentWorldLevelName$(CurrentLevel -CurrentWorld*10,i),10.5+i,1,55+i)
	Next
	If CurrentLevel - CurrentWorld*10<CurrentWorldLevelN
		; place cursor
		If CurrentWorldLevelN>5
			PositionEntity CurrentWorldLevelEntity(12),-.8+(CurrentLevel Mod 5)*.4,.6-Floor((CurrentLevel Mod 10)/5)*.35,-0.01
		Else
			PositionEntity CurrentWorldLevelEntity(12),-.8+(CurrentLevel Mod 5)*.4,.425,-0.01
		EndIf
		; and text
		CenterText("Level "+Str$((CurrentLevel Mod 10)+1),9.5,1,54)
		If CurrentPlayerLevelScore(CurrentLevel)=0
			If(LevelTimer Mod 160 < 120)
				If Language=0
					CenterText("Not Completed Yet",13,1,58)
					CenterText("Press 'Enter' to try this Level.",14,1,59)
				Else If Language=1
					CenterText("Noch nicht komplett.",13,1,58)
					CenterText("'Enter' um das Level zu starten.",14,1,59)
				Else If Language=2
					CenterText("Pas encore complete.",13,1,58)
					CenterText("Tape 'Entree' pour essayer ce niveau.",14,1,59)
	
				EndIf
			Else
				DisplayText("",0,0,1,58)
				DisplayText("",0,0,1,59)
			EndIf
			If Language=0
				DisplayText("Use Cursor Keys to select different Levels.",4,18,1,60)
			Else If Language=1
				DisplayText("Benutze die Pfeiltasten zur Levelauswahl.",5,18,1,60)
			Else If Language=2
				DisplayText("Utilise les fleches pour choisir les niveaux.",3,18,1,60)
	
			EndIf
		Else
			If Language=0
				CenterText("Completed!",13,1,58)
				CenterText("Score: "+Str$(CurrentPlayerLevelScore(CurrentLevel)),14,1,59)
				If(LevelTimer Mod 160 < 120)
					DisplayText("Replay Levels to try and beat your score anytime.",0.5,18,1,60)
				Else
					DisplayText("",0,0,1,60)
				EndIf
			Else If Language=1
				CenterText("Beendet!",13,1,58)
				CenterText("Punkte: "+Str$(CurrentPlayerLevelScore(CurrentLevel)),14,1,59)
				If(LevelTimer Mod 160 < 120)
					DisplayText("Levels koennen jederzeit nochmal gespielt werden.",0.5,18,1,60)
				Else
					DisplayText("",0,0,1,60)
				EndIf
			Else If Language=2
				CenterText("Reussi!",13,1,58)
				CenterText("Score: "+Str$(CurrentPlayerLevelScore(CurrentLevel)),14,1,59)
				If(LevelTimer Mod 160 < 120)
					DisplayText("      Rejoue les niveaux n'importe quand.      ",1,18,1,60)
				Else
					DisplayText("",0,0,1,60)
				EndIf

			EndIf
		EndIf	
	Else If CurrentLevel - CurrentWorld*10=CurrentWorldLevelN
		PositionEntity CurrentWorldLevelEntity(12),1.3,.425,-.01
		If Language=0
			CenterText("Next World",9.5,1,54)
		Else If language=1
			CenterText("Zur naechsten Welt!",9.5,1,54)
		Else If language=2
			CenterText("Monde suivant",9.5,1,54)


		EndIf
		DisplayText("",1,1,1,58)
		DisplayText("",1,1,1,59)
		DisplayText("",1,1,1,60)
	Else 
		PositionEntity CurrentWorldLevelEntity(12),-1.3,.425,-.01
		If Language=0
			CenterText("Previous World",9.5,1,54)
		Else If language=1
			CenterText("Zur vorherigen Welt!",9.5,1,54)
		Else If language=2
			CenterText("Monde precedant",9.5,1,54)

		EndIf

		
		DisplayText("",1,1,1,58)
		DisplayText("",1,1,1,59)
		DisplayText("",1,1,1,60)

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
	

	;how many 'frames' have elapsed	
	TweenTicks=TweenElapsed/TweenPeriod
	
	;fractional remainder
	Tween#=Float(TweenElapsed Mod TweenPeriod)/Float(Tweenperiod)
	
	For k=1 To TweenTicks
		Tweentime=Tweentime+Tweenperiod
		If k=Tweenticks 
			CaptureWorld
			
		EndIf

		
		; do the updates in here
		
		LevelTimer=LevelTimer+1
		PositionTexture MenuBackGroundTex,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		PositionTexture MenuBackGroundTex2,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		TurnEntity CurrentWorldLevelEntity(12),0,3,0
		
		
		
		UpdateWorld
	
	Next

	
	;For k=1 To TweenTicks
	;	Tweentime=Tweentime+Tweenperiod
	;	If k=Tweenticks 
	;		CaptureWorld
	;		
	;	EndIf
;
;		
;		; do the updates in here
;		
;		; scroll water texture
;		Watertexu#=Watertexu#+0.001
;		Watertexv#=Watertexv#+0.0005
;		If Watertexu#=1.0 Then Watertexu#=0.0
;		If Watertexv#=1.0 Then Watertexv#=0.0	
;		If Watertexr#=360.0 Then Watertexr#=0.0
;		PositionTexture WaterTexture,Watertexu#,Watertexv#
;	
;		; rotate camera
;		If CurrentWorldCameraAngle>=2;(CurrentLevel Mod 10)*360/CurrentWorldLevelN	
;			For i=0 To CurrentWorldLevelN-1 +1+Sgn(CurrentWorld)
;				TurnEntity CurrentWorldLevelEntity(i),0,-2,0
;			Next
;			TurnEntity Camera,0,0,-2
;			CurrentWorldCameraAngle=CurrentWorldCameraAngle-2
;		Else If CurrentWorldCameraAngle<=-2;(CurrentLevel Mod 10)*360/CurrentWorldLevelN	
;			For i=0 To CurrentWorldLevelN-1 +1+Sgn(CurrentWorld)
;				TurnEntity CurrentWorldLevelEntity(i),0,2,0
;			Next
;			TurnEntity Camera,0,0,2
;			CurrentWorldCameraAngle=CurrentWorldCameraAngle+2
;		EndIf
;		
;		
;		
;		UpdateWorld
;	
;	Next
		
	RenderWorld tween
	
	Flip
	
	
	; enter	
	If (KeyDown(28) Or KeyDown(156))
		If CursorKeyPressed=False
			CursorKeyPressed=True
			
			If CurrentLevel -CurrentWorld*10<CurrentWorldLevelN
				; enter a level?
				; also get rid of level entities
				For i=0 To CurrentWorldLevelN+Sgn(CurrentWorld)
					FreeEntity CurrentWorldLevelEntity(i)
				Next
				FreeEntity CurrentWorldLevelEntity(12)
				EndMenu()
				
				StartLevel(CurrentLevel)
				PlayMySound(SFXTalk(Rand(0,3)),0)
				Return
			Else If CurrentLevel -CurrentWorld *10=CurrentWorldLevelN
				; Next World
				NextWorldFlag=0
				For j=0 To CurrentWorld
					For i=0 To 9
						If WorldLevelName$(j*10+i)<>"" And CurrentPlayerLevelScore(j*10+i)=0 Then NextWorldFlag=1
					Next
				Next

				If NextWorldFlag=0 Or (CurrentWorld<9 And LockWorlds=1)

					
					; also get rid of level entities
					For i=0 To CurrentWorldLevelN+Sgn(CurrentWorld)
						FreeEntity CurrentWorldLevelEntity(i)
					Next
					FreeEntity CurrentWorldLevelEntity(12)
					;onward and upward!
					CurrentWorld=CurrentWorld+1
					If CurrentWorld=7 And Version=12
						CurrentWorld=9
					EndIf

					EndMenu()
					
					CurrentLevel=CurrentWorld*10
					
					If (CurrentWorld=2 And Version<10)
						; end of demo
						StartMenu(11)
						Return
					EndIf
					If CurrentWorld=10
						; end of game
						StartMenu(12)
						Return
					EndIf
					
					StartLevelMusic()
					
					StartMenu(4)
					Return
				Else
					PlayMySound(SFXTalk(4),0)
				EndIf
			Else 
				; Previous World
				; also get rid of level entities
				For i=0 To CurrentWorldLevelN+Sgn(CurrentWorld)
					FreeEntity CurrentWorldLevelEntity(i)
				Next
				FreeEntity CurrentWorldLevelEntity(12)
				CurrentWorld=CurrentWorld-1
				If CurrentWorld=8 And Version=12
					CurrentWorld=6
				EndIf
				EndMenu()
				
				CurrentLevel=CurrentWorld*10
				
				StartLevelMusic()

				StartMenu(4)
				Return
			EndIf
			
			Return
		EndIf
	; cursor keys
	Else If KeyDown(75) Or KeyDown(203)
		If CursorKeyPressed=False 
			PlayMySound(SFXMenuSound(0),0)	
			
			CursorKeyPressed=True
			CurrentLevel=CurrentLevel-1
		
			If CurrentLevel<CurrentWorld*10
				CurrentLevel=CurrentWorld*10+CurrentWorldLevelN+Sgn(CurrentWorld)
			EndIf
		EndIf
	
	Else If KeyDown(77) Or KeyDown(205)

		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			
			CursorKeyPressed=True
			CurrentLevel=CurrentLevel+1
	
			If CurrentLevel>=CurrentWorld*10+CurrentWorldLevelN +1+Sgn(CurrentWorld)
				CurrentLevel=CurrentWorld*10
			EndIf
		EndIf
	Else If KeyDown(72) Or KeyDown(200)
		If CursorKeyPressed=False And CurrentWorldLevelN>5 
			PlayMySound(SFXMenuSound(0),0)	
			
			CursorKeyPressed=True
			CurrentLevel=CurrentLevel-5
		
			If CurrentLevel<CurrentWorld*10
				CurrentLevel=CurrentWorld*10+CurrentWorldLevelN-1
			EndIf
		EndIf
	Else If KeyDown(80) Or KeyDown(208)
		If CursorKeyPressed=False And CurrentWorldLevelN>5
			PlayMySound(SFXMenuSound(0),0)	
			
			CursorKeyPressed=True
			CurrentLevel=CurrentLevel+5
		
			If CurrentLevel>CurrentWorld*10+CurrentWorldLevelN-1
				CurrentLevel=CurrentWorld*10+CurrentWorldLevelN-1
			EndIf
		EndIf
	Else If KeyDown(1)
		If CursorKeyPressed=False
			CursorKeyPressed=True
			; also get rid of level entities
			For i=0 To CurrentWorldLevelN+Sgn(CurrentWorld)
				FreeEntity CurrentWorldLevelEntity(i)
			Next
			FreeEntity CurrentWorldLevelEntity(12)
			EndMenu()
			StartMenu(1)
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
	
	Flip
	
	
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False	
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=3
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
			KillStinky()
			If LoofExists=True Then KillLoof()
			GamePhase=1
		Else If CurrentMenuOption=1
			; Back to Menu
			EndLevel()
			If CurrentLevel<PlayGroundLevel
				StartMenu(4)
			Else
				StartMenu(8)
			EndIf
			
		Else If CurrentMenuOption=2
			; Exit Game
			EndLevel()
			StartMenu(1)
		
		Else If CurrentMenuOption=3
			; Resume Game
					
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
	ScaleEntity MenuSelector,4.5*.012,.02,0.001
	

	
	
	
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
		PositionTexture MenuBackGroundTex,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		PositionTexture MenuBackGroundTex2,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
	
		
		
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

Function MenuLoop7()

	; Display Static Text
	
	If Version>=10
		i=1
	Else 
		i=0
	EndIf
	
	If Language=0
		CenterText("Welcome to the Playground",2+i,1,50)
		CenterText("-------------------------",3+i,1,51)
		CenterText("The Playground offers a collection",5+i,1,52)
		CenterText("of levels specifically designed for",6+i,1,53)
		CenterText("very young or less experienced players.",7+i,1,54)
		CenterText("Rather than placing the emphasis",8+i,1,55)
		CenterText("on progressively challenging trials,",9+i,1,56)
		CenterText("the Playground levels focus on exploration,",10+i,1,57)
		CenterText("discovery, and simple puzzles.",11+i,1,58)
	Else If Language=1
		CenterText("Beginner Levels",2+i,1,50)
		CenterText("---------------",3+i,1,51)
		CenterText("Diese Levels bieten eine Auswahl an",5+i,1,52)
		CenterText("speziell entworfenen Levels fuer sehr",6+i,1,53)
		CenterText("Junge oder unerfahrene Spieler.",7+i,1,54)
		CenterText("Im Gegensatz zu den Herausforderungen",8+i,1,55)
		CenterText("der anderen Levels, konzentrieren sich",9+i,1,56)
		CenterText("die Beginner Levels auf das Training,",10+i,1,57)
		CenterText("Entdeckung und einfache Aufgaben.",11+i,1,58)
	Else If Language=2
		CenterText("BINVENUE AU TERRAIN DE JEU",2+i,1,50)
		CenterText("--------------------------",3+i,1,51)
		CenterText("Le terrain de jeu offre une collection",5+i,1,52)
		CenterText("de niveaux specialement penses pour les",6+i,1,53)
		CenterText("tres jeunes ou les joueurs moins experimentes.",7+i,1,54)
		CenterText("Au lieu de mettre l'emphase sur",8+i,1,55)
		CenterText("des tentatives de plus en plus exigentes,",9+i,1,56)
		CenterText("ces niveaux visent l'exploration, la",10+i,1,57)
		CenterText("decouverte et les casse-tetes simples.",11+i,1,58)

	EndIf
	
	
	
	
	If Version>=10
		If Language=0 Then CenterText("Also contains Five Bonus Levels.",13+i,1,59)

		i=1
		If Language=0
			CenterText("HERE WE GO!",15+i,1,63)
			CenterText("NO THANKS",16+i,1,64)
		Else If Language=1
			CenterText("LOS GEHTS!",15+i,1,63)
			CenterText("NEIN DANKE",16+i,1,64)
		Else If Language=2
			CenterText("ALLONS-Y",15+i,1,63)
			CenterText("NON MERCI",16+i,1,64)


		EndIf
	Else 
		If Language=0
			If Leveltimer Mod 300 < 250
				CenterText("The Playground is not available in the free",13,1,59)
				CenterText("demo version. Please register to access",14,1,60)
				CenterText("these and all other levels.",15,1,62)
			Else
				CenterText("",13,1,59)
				CenterText("",14,1,60)
				CenterText("",15,1,62)
			EndIf
			CenterText("PRESS ENTER",17,1,63)
		Else If Language=1
			If Leveltimer Mod 300 < 250
				CenterText("Die Beginner Levels sind nicht in der",13,1,59)
				CenterText("Demo erhaeltlich. Bitte kaufen Sie die",14,1,60)
				CenterText("Vollversion um diese Levels zu spielen.",15,1,62)
			Else
				CenterText("",13,1,59)
				CenterText("",14,1,60)
				CenterText("",15,1,62)
			EndIf
			CenterText("'ENTER'",17,1,63)
		Else If Language=2
			If Leveltimer Mod 300 < 250
				CenterText("Le terrain de jeu n'est pas disponible",13,1,59)
				CenterText("avec la version demo gratuit. S.V.P",14,1,60)
				CenterText("enregistrez-vous pour y acceder.",15,1,62)
			Else
				CenterText("",13,1,59)
				CenterText("",14,1,60)
				CenterText("",15,1,62)
			EndIf
			CenterText("TAPE 'ENTREE'",17,1,63)

		EndIf
		i=4
	EndIf


	
	
	
	
		
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(15+i+CurrentMenuOption)*.044+.027,1.1
	ScaleEntity MenuSelector,11.5*.012,.02,0.001
	

	
	
	
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
		PositionTexture MenuBackGroundTex,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		PositionTexture MenuBackGroundTex2,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
	
		
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	
	
	If (KeyDown(200) Or KeyDown(72)) And Version>=10
		If CursorKeyPressed=False		
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=1
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) And Version>=10
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
			If CurrentMenuOption=0 And Version>=10
				PlayMySound(SFXTalk(1),0)
				EndMenu()
				StartMenu(8)
			Else
				EndMenu()
				StartMenu(1)
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

Function MenuLoop8()

	; *******************
	
	; FIRST: load up the world data, i.e. each levelname, score, world name
	; *******************
	;DisplayText(CurrentPlayerName$,0,0,1,61)
	If Language=0
		CenterText("WELCOME TO THE PLAYGROUND",2,1,50)
	Else If Language=1
		CenterText("BEGINNER LEVELS",2,1,50)
	Else If Language=2
		CenterText("BIENVENUE AU TERRAIN DE JEU",2,1,50)

	EndIf
	;CenterText(WorldName$(CurrentWorld),3,1,51)
	;For i=0 To 499
	;	If CurrentPlayerLevelScore(i)>0
	;		j=j+CurrentPlayerLevelScore(i)
	;	EndIf
	;Next

	;DisplayText("TOTAL SCORE:",38,0,1,52)
	;DisplayText(Str$(j),50-Len(Str$(j)),1,1,53)	
	
			
	
	For i=0 To 2
		CenterText(CurrentWorldLevelName$(CurrentLevel -CurrentWorld*10,i),10.5+i,1,55+i)
	Next
	
	; place cursor
	If CurrentWorldLevelN>5
		PositionEntity CurrentWorldLevelEntity(12),-.8+(CurrentLevel Mod 5)*.4,.6-Floor((CurrentLevel Mod 10)/5)*.35,-0.01
	Else
		PositionEntity CurrentWorldLevelEntity(12),-.8+(CurrentLevel Mod 5)*.4,.425,-0.01
	EndIf
	; and text
	CenterText("Level "+Str$((CurrentLevel Mod 10)+1),9.5,1,54)

	If Language=0
		CenterText("Press 'Enter' to try this Level.",14,1,59)
		CenterText("Use Cursor Keys to select different Levels.",17,1,60)
		CenterText("Levels can be replayed anytime.",18,1,60)
	Else If Language=1
		CenterText("'Enter' um dieses Level zu spielen.",14,1,59)
		CenterText("Pfeiltasten um andere Levels auszuwaehlen.",17,1,60)
		CenterText("",18,1,60)
	Else If Language=2
		CenterText("Tape 'Entree' pour essayer ce niveau.",14,1,59)
		CenterText("Utilise les fleches pour choisir les niveaux.",17,1,60)
		CenterText("",18,1,60)

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
		PositionTexture MenuBackGroundTex,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		PositionTexture MenuBackGroundTex2,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		TurnEntity CurrentWorldLevelEntity(12),0,3,0
		
		
		
		UpdateWorld
	
	Next

	
	;For k=1 To TweenTicks
	;	Tweentime=Tweentime+Tweenperiod
	;	If k=Tweenticks 
	;		CaptureWorld
	;		
	;	EndIf
;
;		
;		; do the updates in here
;		
;		; scroll water texture
;		Watertexu#=Watertexu#+0.001
;		Watertexv#=Watertexv#+0.0005
;		If Watertexu#=1.0 Then Watertexu#=0.0
;		If Watertexv#=1.0 Then Watertexv#=0.0	
;		If Watertexr#=360.0 Then Watertexr#=0.0
;		PositionTexture WaterTexture,Watertexu#,Watertexv#
;	
;		; rotate camera
;		If CurrentWorldCameraAngle>=2;(CurrentLevel Mod 10)*360/CurrentWorldLevelN	
;			For i=0 To CurrentWorldLevelN-1 +1+Sgn(CurrentWorld)
;				TurnEntity CurrentWorldLevelEntity(i),0,-2,0
;			Next
;			TurnEntity Camera,0,0,-2
;			CurrentWorldCameraAngle=CurrentWorldCameraAngle-2
;		Else If CurrentWorldCameraAngle<=-2;(CurrentLevel Mod 10)*360/CurrentWorldLevelN	
;			For i=0 To CurrentWorldLevelN-1 +1+Sgn(CurrentWorld)
;				TurnEntity CurrentWorldLevelEntity(i),0,2,0
;			Next
;			TurnEntity Camera,0,0,2
;			CurrentWorldCameraAngle=CurrentWorldCameraAngle+2
;		EndIf
;		
;		
;		
;		UpdateWorld
;	
;	Next
		
	RenderWorld tween
	
	Flip
	
	
	; enter	
	If (KeyDown(28) Or KeyDown(156))
		If CursorKeyPressed=False
			CursorKeyPressed=True
			
			If CurrentLevel -CurrentWorld*10<CurrentWorldLevelN
				; enter a level?
				; also get rid of level entities
				For i=0 To CurrentWorldLevelN-1
					FreeEntity CurrentWorldLevelEntity(i)
				Next
				FreeEntity CurrentWorldLevelEntity(12)
				EndMenu()
				
				StartLevel(CurrentLevel)
				PlayMySound(SFXTalk(Rand(0,3)),0)
			
			EndIf
			
			Return
		EndIf
	; cursor keys
	Else If KeyDown(75) Or KeyDown(203)
		If CursorKeyPressed=False 
			PlayMySound(SFXMenuSound(0),0)	
			
			CursorKeyPressed=True
			CurrentLevel=CurrentLevel-1
		
			If CurrentLevel<CurrentWorld*10
				CurrentLevel=CurrentWorld*10+CurrentWorldLevelN-1
			EndIf
		EndIf
	
	Else If KeyDown(77) Or KeyDown(205)

		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			
			CursorKeyPressed=True
			CurrentLevel=CurrentLevel+1
	
			If CurrentLevel>=CurrentWorld*10+CurrentWorldLevelN
				CurrentLevel=CurrentWorld*10
			EndIf
		EndIf
	Else If KeyDown(72) Or KeyDown(200)
		If CursorKeyPressed=False And CurrentWorldLevelN>5 
			PlayMySound(SFXMenuSound(0),0)	
			
			CursorKeyPressed=True
			CurrentLevel=CurrentLevel-5
		
			If CurrentLevel<CurrentWorld*10
				CurrentLevel=CurrentWorld*10+CurrentWorldLevelN-1
			EndIf
		EndIf
	Else If KeyDown(80) Or KeyDown(208)
		If CursorKeyPressed=False And CurrentWorldLevelN>5
			PlayMySound(SFXMenuSound(0),0)	
			
			CursorKeyPressed=True
			CurrentLevel=CurrentLevel+5
		
			If CurrentLevel>CurrentWorld*10+CurrentWorldLevelN-1
				CurrentLevel=CurrentWorld*10+CurrentWorldLevelN-1
			EndIf
		EndIf
	Else If KeyDown(1)
		If CursorKeyPressed=False
			CursorKeyPressed=True
			; also get rid of level entities
			For i=0 To CurrentWorldLevelN-1
				FreeEntity CurrentWorldLevelEntity(i)
			Next
			FreeEntity CurrentWorldLevelEntity(12)
			EndMenu()
			StartMenu(1)
			Return
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
	;	OptionsResText$="Resolution: "+Str$(MyGfxModeWidth(MyGfxMode))+"x"+Str$(MyGfxModeHeight(MyGfxMode))
		If DetailLevel=5
			OptionsDetailText$="Detail Level: High"
		Else If DetailLevel=3
			OptionsDetailText$="Detail Level: Medium"
		Else If DetailLevel=1
			OptionsDetailText$="Detail Level: Low"
		EndIf
	;	If MyWindowed=0
	;		OptionsWindowedText$="Full Screen"
	;	Else
	;		OptionsWindowedText$="Windowed"
	;	EndIf
		If NoTimeLimit=1
			OptionsTimeLimitText$="No Time Limit"
		Else
			OptionsTimeLimitText$="Timed Mode"
		EndIf
		If LockWorlds=0
			OptionsLockWorldText$="Normal World Progression"
		Else
			OptionsLockWorldText$="All Worlds Unlocked"
		EndIf
	
		OptionsLanguageText$="English"
		
	
	
		; Options Menu
		
			
		CenterText("OPTIONS",5,1,50)
		CenterText("-------",6,1,51)
		CenterText(OptionsSoundText$,8,1,52)
		CenterText(OptionsMusicText$,9,1,53)
		;CenterText(OptionsResText$,10,1,54)
		CenterText(OptionsDetailText$,10,1,55)
		;CenterText(OptionsWindowedText$,12,1,56)
		CenterText(OPtionsLanguageText$,11,1,58)
		CenterText(OPtionsTimeLimitText$,12,1,60)
		If Version>=10 CenterText(OPtionsLockWorldText$,13,1,66)
		CenterText("Visit Us Online",13+(Version/10),1,59)
		CenterText("MAIN MENU",14+(Version/10),1,57)
		
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

		If LockWorlds=0
			OptionsLockWorldText$="Welten Geschlossen"
		Else
			OptionsLockWorldText$="Alle Welten Offen"
		EndIf
	
		
	
		; Options Menu
		
			
		CenterText("OPTIONEN",5,1,50)
		CenterText("--------",6,1,51)
		CenterText(OptionsSoundText$,8,1,52)
		CenterText(OptionsMusicText$,9,1,53)
		;CenterText(OptionsResText$,10,1,54)
		CenterText(OptionsDetailText$,10,1,55)
		;CenterText(OptionsWindowedText$,12,1,56)
		CenterText(OPtionsLanguageText$,11,1,58)
		CenterText(OPtionsTimeLimitText$,12,1,60)
	;	CenterText(OPtionsLockWorldText$,13,1,66)
		If Version>=10 CenterText(OPtionsLockWorldText$,13,1,66)
		CenterText("Unsere Webseite",13+(Version/10),1,59)
		CenterText("HAUPTMENUE",14+(Version/10),1,57)

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

		If LockWorlds=0
			OptionsLockWorldText$="Mondes Ferme"
		Else
			OptionsLockWorldText$="Mondes Ouverte"
		EndIf
	
				
	
	
		; Options Menu
		
			
		CenterText("OPTIONS",5,1,50)
		CenterText("-------",6,1,51)
		CenterText(OptionsSoundText$,8,1,52)
		CenterText(OptionsMusicText$,9,1,53)
		;CenterText(OptionsResText$,10,1,54)
		CenterText(OptionsDetailText$,10,1,55)
		;CenterText(OptionsWindowedText$,12,1,56)
		CenterText(OPtionsLanguageText$,11,1,58)
		CenterText(OPtionsTimeLimitText$,12,1,60)
		If Version>=10 CenterText(OPtionsLockWorldText$,13,1,66)
		CenterText("Notre Site Web",13+(Version/10),1,59)
		CenterText("Menu Principal",14+(Version/10),1,57)

		

	EndIf
	
	
		
	
		
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(8+CurrentMenuOption)*.044+.027,1.1
	ScaleEntity MenuSelector,23*.012,.02,0.001
	
	
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
		PositionTexture MenuBackGroundTex,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		PositionTexture MenuBackGroundTex2,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
	
		
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	
	
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False		
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=6+(Version/10)
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption+1
			If CurrentMenuOption=7+(Version/10) Then CurrentMenuOption=0
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
			

			Else If CurrentMenuOption=2
				If DetailLevel=5
					DetailLevel=3
					
				Else If DetailLevel=3
					DetailLevel=1
				Else 
					DetailLevel=5
				EndIf
				
			
			Else If CurrentMenuOption=3
				Language=Language+1
				If Language=3 Then Language=0
			Else If  CurrentMenuOption=4
				NoTimeLimit=1-NoTimeLimit
				
			Else If  CurrentMenuOption=5 And Version>=10
				LockWorlds=1-LockWorlds

			Else If (CurrentMenuOption=6 And Version>=10) Or (CurrentMenuOption=5 And Version<10)
				EndMenu()
				; go to site!
				VersionNo1=-998
				GameExit=True
				
			Else If (CurrentMenuOption=7  And Version>=10) Or (CurrentMenuOption=6 And Version<10)

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
	
	
	
		
	
	
	
		
	PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(10+CurrentMenuOption)*.044+.027,1.1
	ScaleEntity MenuSelector,11.5*.012,.02,0.001
	

	
	
	
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
		PositionTexture MenuBackGroundTex,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		PositionTexture MenuBackGroundTex2,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
	
		
		
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
				If Version>=10
					EndMenu()
					GameExit=True
				Else 
					EndMenu()
					StartMenu(11)
				EndIf
			Else
				EndMenu()
				StartMenu(1)
			EndIf
			
		EndIf
	Else If KeyDown(1)
		If CursorKeyPressed=False
			CursorKeyPressed=True
			If Version>=10
				EndMenu()
				GameExit=True
			Else 
				EndMenu()
				StartMenu(11)
			EndIf
		EndIf

	Else
		CursorKeyPressed=False
	EndIf


	
End Function

Function MenuLoop11()

	; Display Static Text
	Multiplier=4
		
	
		
	
	If EndMenuStage=0	
		CenterText("THANK YOU FOR PLAYING THE WONDERLAND DEMO",1,1,50)
		CenterText("-----------------------------------------",2,1,51)
		
		DisplayText("FULL VERSION",32,13,1,52)
		DisplayText("SOME REVIEWS",32,14,1,53)
		DisplayText(" GET IT NOW",32,15,1,54)
		DisplayText(" NO, THANKS",32,16,1,55)
	Else
		CenterText("",1,1,50)
		CenterText("",2,1,51)
	
		DisplayText("!! FREE BONUS !!",30,13,1,52)
		DisplayText("OTHER GAMES",32,14,1,53)
		DisplayText("EXIT TO WINDOWS",30.5,15,1,54)
		DisplayText("",30.5,15,1,55)
	EndIf

	
	PositionEntity MenuSelector,.274,0.372-Float(13+CurrentMenuOption)*.044+.027,1.1
	ScaleEntity MenuSelector,16*.012,.02,0.001
	
	speed#=80.0
	t=DemoEndTextTimer(CurrentMenuOption)
	y#=Float(t Mod speed)/speed
	If EndMenuStage=0 And (CurrentMenuOption=1 Or CurrentMenuOption=0)
		
		t2=Floor(t/speed)
		While t2>=(38+20*CurrentMenuOption)
			t2=t2-(38+20*CurrentMenuOption)
		Wend
		For i=6 To 18
			If i=6
				DisplayText(DemoEndText$(CurrentMenuOption,t2),1,i-y,1.0-y,52+i)
			Else If i=18
				DisplayText(DemoEndText$(CurrentMenuOption,t2),1,i-y,y,52+i)
			Else
				DisplayText(DemoEndText$(CurrentMenuOption,t2),1,i-y,1,52+i)
			EndIf
			
			;DemoEndText$(CurrentMenuOption,t2)
			t2=t2+1
			If (t2>=(38+20*CurrentMenuOption)) Then t2=0
			
		Next
		

		
	
	
	EndIf
	
	If EndMenuStage=0
		If CurrentMenuOption=0
	;		DemoEndTextTimer(0)=DemoEndTextTimer(0)+1
			DisplayText("HERE ARE ONLY SOME OF THE ",1,3.5,1,56)
			DisplayText("MANY FULL VERSION FEATURES:",1,4.5,1,57)
			
		Else If CurrentMenuOption=1
	;		DemoEndTextTimer(1)=DemoEndTextTimer(1)+1
			DisplayText("  ONE OF THE BEST REVIEWED  ",0,3.5,1,56)
			DisplayText("ACTION/PUZZLERS OF THE YEAR!",0,4.5,1,57)
			
		Else If CurrentMenuOption=2
			DisplayText("   GET THE FULL VERSION   ",1.5,4,1,56)
			DisplayText("    OF WONDERLAND NOW!    ",1.5,5,1,57)
			DisplayText(" Press 'Enter' to connect ",1.5,7,1,58)
			DisplayText("   to our secure online   ",1.5,8,1,59)
			DisplayText("      order page now.      ",1.5,9,1,60)
			DisplayText("   Or visit us online at   ",1,11,1,61)
			DisplayText("  WWW.MIDNIGHTSYNERGY.COM  ",1,12,1,62)
			DisplayText("  We thank you in advance  ",1,15,1,63)
			DisplayText("     for your support!     ",1,16,1,64)
			DisplayText("",1,15,1,65)
			DisplayText("",1,17,1,66)
			For i=67 To 70
				displayText("",1,1,1,i)
			Next


		Else
			DisplayText("  THANK YOU FOR PLAYING!  ",1.5,4,1,56)
			DisplayText("",1,5,1,57)
			DisplayText("   Remember to visit us   ",1.5,5.5,1,58)
			DisplayText("       online to get       ",1,6.5,1,59)
			DisplayText("* 85 Levels in 10 Worlds *",1.5,8,1,60)
			DisplayText("* New Game-Play Elements *",1.5,9,1,61)
			DisplayText("* New Graphics and Music *",1.5,10,1,62)
			DisplayText("  And of course all games  ",1,11.5,1,63)
			DisplayText("  come with our risk-free  ",1.5,12.5,1,64)
			DisplayText("  money-back-guarantee!!!  ",1,13.5,1,65)
			DisplayText("  WWW.MIDNIGHTSYNERGY.COM  ",1,16,1,66)
			For i=67 To 70
				displayText("",1,1,1,i)
			Next

		EndIf
	EndIf
	If EndMenuStage=1
		If CurrentMenuOption=0
			Centertext("UNLOCK TWO FREE (!!) EXTRA LEVELS IN THIS DEMO!",1,1,56)
			Centertext("-----------------------------------------------",2,1,57)
			Centertext("Sign-up to our mailing list by sending an e-mail",3.5,1,58)
			Centertext("to BONUS@MIDNIGHTSYNERGY.COM",4.5,1,59)
			Centertext("An automated reply e-mail will include a SECRET",6,1,60)
			Centertext("CODE to unlock TWO NEW LEVELS in the demo, free!",7,1,61)
			Centertext("You will also receive notices when new free games,",9,1,62)
			Centertext("demos, and updates are available. Of course, you",10,1,63)
			Centertext("can unsubscribe from this service at any time.",11,1,64)
			For i=65 To 70
				displayText("",1,1,1,i)
			Next


		
		Else If CurrentMenuOption=1
			Centertext("INTERESTED IN OTHER FREE GAME DEMOS?",1,1,56)
			Centertext("------------------------------------",2,1,63)
			Centertext("Visit us online at www.midnightsynergy.com",4,1,57)
			Centertext("for free demos of our award-winning games,",5,1,58)
			Centertext("including the retro classic 'Intensity XS',",6,1,59)
			Centertext("the arcade puzzler 'Frazzled', the strategy",7,1,60)
			Centertext("game 'Colony', and more.",8,1,61)
			Centertext("Press 'Enter' to connect to our games site now!",10,1,62)
			For i=64 To 70
				displayText("",1,1,1,i)
			Next

		Else If CurrentMenuOption=2
			Centertext("Thank you for playing the Wonderland Demo!",5,1,56)
			Centertext("We hope to see you again soon!",8,1,57)
			For i=58 To 70
				displayText("",1,1,1,i)
			Next

			
		EndIf
	EndIf
	
	


	


			
	
	
	;pictures
	If EndMenuStage=0
		ScaleEntity TitleChar(3),.7+.05*Sin((LevelTimer)),.7+.05*Sin((LevelTimer)),0.001
		ScaleEntity TitleChar(1),.7+.05*Sin((LevelTimer)),.7+.05*Sin((LevelTimer)),0.001
		ScaleEntity TitleChar(2),.7+.05*Sin((LevelTimer)),.7+.05*Sin((LevelTimer)),0.001
		RotateEntity TitleChar(1),0,0,15*Sin((LevelTimer*2/3))
		If LevelTimer Mod (360*9) < 360
			PositionEntity TitleChar(3),3-2*Sin((LevelTimer Mod 360)/4),.4,0
		Else If LevelTimer Mod (360*9) <2*360
		
		Else If LevelTimer Mod (360*9) <3*360
			PositionEntity TitleChar(3),3-2*Cos((LevelTimer Mod 360)/4),.4,0
		Else If LevelTimer Mod (360*9) < 4*360
			PositionEntity TitleChar(1),3-2*Sin((LevelTimer Mod 360)/4),.4,0
			
		Else If LevelTimer Mod (360*9) <5*360
		
		Else If LevelTimer Mod (360*9) <6*360
			PositionEntity TitleChar(1),3-2*Cos((LevelTimer Mod 360)/4),.4,0
		Else If LevelTimer Mod (360*9) < 7*360
			PositionEntity TitleChar(2),3-2*Sin((LevelTimer Mod 360)/4),.4+.1*Sin(leveltimer),0
			
		Else If LevelTimer Mod (360*9) <8*360
			PositionEntity TitleChar(2),1,.4+.1*Sin(leveltimer),0
		Else If LevelTimer Mod (360*9) <9*360
			PositionEntity TitleChar(2),3-2*Cos((LevelTimer Mod 360)/4),.4+.1*Sin(leveltimer),0
		EndIf
	Else
		For i=1 To 3
			PositionEntity TitleChar(i),5,5,0
		Next
		ScaleEntity TitleChar(0),.5,.5,.5
		
		If LevelTimer<360
			PositionEntity TitleChar(0),-.86,-2+Sin(Leveltimer/4),0
		Else
			PositionEntity TitleChar(0),-.86,-1,0
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
		
		If EndMenuStage=0
			If CurrentMenuOption=0
				DemoEndTextTimer(0)=DemoEndTextTimer(0)+1
			EndIf
			If CurrentMenuOption=1
				DemoEndTextTimer(1)=DemoEndTextTimer(1)+1
			EndIf
		EndIf


		PositionTexture MenuBackGroundTex,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		PositionTexture MenuBackGroundTex2,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
	
		
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	
	
	Flip

	
	If (KeyDown(200) Or KeyDown(72)) 
		If CursorKeyPressed=False		
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption-1
			If CurrentMenuOption=-1 Then CurrentMenuOption=3-EndMenuStage
			CursorKeyPressed=True
		EndIf

	Else If (KeyDown(208) Or KeyDown(80)) 
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CurrentMenuOption=CurrentMenuOption+1
			If CurrentMenuOption=4-EndMenuStage Then CurrentMenuOption=0
			CursorKeyPressed=True
		EndIf
	Else If (KeyDown(28) Or KeyDown(156))
		If CursorKeyPressed=False
			PlayMySound(SFXMenuSound(0),0)	
			CursorKeyPressed=True
			If EndMenuStage=0
				If CurrentMenuOption=3 
					If IsExtraDemoLevelActive=0
						CurrentMenuOption=0
						EndMenuStage=1
						LevelTimer=0
					Else
						EndMenu()
						GameExit=True
					EndIf
				Else
					EndMenu()
					VersionNo1=-1
					GameExit=True
				EndIf
			Else
				If CurrentMenuOption=0
					EndMenu()
					VersionNo1=-3
					GameExit=True
				Else If CurrentMenuOption=1
					EndMenu()
					VersionNo1=-2
					GameExit=True
				Else If CurrentMenuOption=2 
					EndMenu()
					GameExit=True
				EndIf

			
			EndIf
			
		EndIf
	Else If KeyDown(1)
		If CursorKeyPressed=False
			CursorKeyPressed=True
			EndMenu()
			VersionNo1=-998
			GameExit=True
		EndIf

	Else
		CursorKeyPressed=False
	EndIf
	


	
End Function

Function MenuLoop11old()

	; Display Static Text
	
	If Language=0	
		CenterText("THANK YOU FOR PLAYING THE WONDERLAND DEMO",1,1,50)
		CenterText("-----------------------------------------",2,1,51)
		CenterText("THE FULL VERSION INCLUDES:",3,1,60)
		
		Multiplier=4
		
		
		
		If LevelTimer>60*Multiplier
			DisplayText("EIGHTY-FIVE LEVELS",3,5,1,52)
		EndIf
		If LevelTimer>100*Multiplier
			DisplayText(" OVER TEN WORLDS!!!",3,6,1,53)
		EndIf
		If LevelTimer>210*Multiplier
			DisplayText("NEW GAMEPLAY ELEMENTS:",24,8,1,54)
		EndIf
		If LevelTimer>250*Multiplier
			DisplayText(" ICE, LAVA, Z-BOTS, TIMERS,",24,9,1,55)
		EndIf
		If LevelTimer>290*Multiplier
			DisplayText("  AND MUCH, MUCH MORE!!!",24,10,1,56)
		EndIf
		If LevelTimer>400*Multiplier
			DisplayText("MORE GRAPHICS, MORE MUSIC,",3,12,1,57)
		EndIf
		If LevelTimer>440*Multiplier
			DisplayText(" MORE PUZZLES, MORE ACTION,",3,13,1,58)
		EndIf
		If LevelTimer>480*Multiplier
			DisplayText("  MORE EVERYTHING!!!",3,14,1,59)
		EndIf
	Else If Language=1
		CenterText("VIELEN DANK FUER'S SPIELEN",1,1,50)
		CenterText("--------------------------",2,1,51)
		CenterText("IN DER VOLLVERSION GIBT ES:",3,1,60)
		
		Multiplier=4
		
		
		
		If LevelTimer>60*Multiplier
			DisplayText("85 LEVELS",3,5,1,52)
		EndIf
		If LevelTimer>100*Multiplier
			DisplayText(" IN 10 WELTEN!!!",3,6,1,53)
		EndIf
		If LevelTimer>210*Multiplier
			DisplayText("NEUE LEVELELEMENTE:",24,8,1,54)
		EndIf
		If LevelTimer>250*Multiplier
			DisplayText(" EIS, LAVA, Z-BOTS,",24,9,1,55)
		EndIf
		If LevelTimer>290*Multiplier
			DisplayText("  UND VIEL, VIEL MEHR!!!",24,10,1,56)
		EndIf
		If LevelTimer>400*Multiplier
			DisplayText("NEUE GRAPHIKEN, NEUE MUSIK,",3,12,1,57)
		EndIf
		If LevelTimer>440*Multiplier
			DisplayText(" MEHR PUZZLES, MEHR ACTION,",3,13,1,58)
		EndIf
		If LevelTimer>480*Multiplier
			DisplayText("  MEHR VON ALLEM!!!",3,14,1,59)
		EndIf
	Else If Language=2
		CenterText("MERCI DE JOUER AU DEMO DE WONDERLAND",1,1,50)
		CenterText("------------------------------------",2,1,51)
		CenterText("LA VERSION COMPLETE COMPREND:",3,1,60)
		
		Multiplier=4
		
		
		
		If LevelTimer>60*Multiplier
			DisplayText("85 NIVEAUX",3,5,1,52)
		EndIf
		If LevelTimer>100*Multiplier
			DisplayText(" DANS 10 MONDES!!!",3,6,1,53)
		EndIf
		If LevelTimer>210*Multiplier
			DisplayText("DE NOUVEAUX ELEMENTS DE JEU:",22,8,1,54)
		EndIf
		If LevelTimer>250*Multiplier
			DisplayText(" GLACE, LAVE, ZI-BOTS,",22,9,1,55)
		EndIf
		If LevelTimer>290*Multiplier
			DisplayText("  ET BIEN PLUS ENCORE!!!",22,10,1,56)
		EndIf
		If LevelTimer>400*Multiplier
			DisplayText("PLUS DE DECORS, DE MUSIQUES,",3,12,1,57)
		EndIf
		If LevelTimer>440*Multiplier
			DisplayText(" DE CASSE-TETE ET D'ACTION,",3,13,1,58)
		EndIf
		If LevelTimer>480*Multiplier
			DisplayText("  PLUS DE TOUT!!!",3,14,1,59)
		EndIf
	
		
	EndIf
	
	If LevelTimer<30
		PositionEntity TitleChar(3),.7,.5,0
		ScaleEntity TitleChar(3),0,0,0.001
		
		PositionEntity TitleChar(1),-1,-0.1,0
		ScaleEntity TitleChar(1),0,0,0.001
	EndIf
		

	
	If LevelTimer>60*Multiplier And LevelTimer<120*Multiplier
		ScaleEntity TitleChar(3),.5*(LevelTimer-60*Multiplier)/(60*Multiplier),.5*(LevelTimer-60*Multiplier)/(60*Multiplier),0.001
	Else If LevelTimer>120*Multiplier
		ScaleEntity TitleChar(3),.5+.1*Sin((LevelTimer-120*Multiplier)),.5+.1*Sin((LevelTimer-120*Multiplier)),0.001
	EndIf
	If LevelTimer>210*Multiplier And LevelTimer<270*Multiplier
		ScaleEntity TitleChar(1),.5*(LevelTimer-210*Multiplier)/(60*Multiplier),.5*(LevelTimer-210*Multiplier)/(60*Multiplier),0.001
	EndIf
	If LevelTimer>210*Multiplier
		RotateEntity TitleChar(1),0,0,20*Sin((LevelTimer-210*Mutliplier))
	EndIf
	If LevelTimer>400*Multiplier And LevelTimer<460*Multiplier
		PositionEntity TitleChar(2),1+.2*Sin(LevelTimer-400*Multiplier),-0.75,0
		ScaleEntity TitleChar(2),.5*(LevelTimer-400*Multiplier)/(60*Multiplier),.5*(LevelTimer-400*Multiplier)/(60*Multiplier),0.001
	EndIf
	If LevelTimer>460*Multiplier
		PositionEntity TitleChar(2),1+.2*Sin(LevelTimer-400*Multiplier),-0.75,0
		ScaleEntity TitleChar(2),.5+.05*Sin((LevelTimer-460*Multiplier)),.5+.05*Sin((LevelTimer-460*Multiplier)),0.001
	EndIf

	

	





	

	
	
	
	
	If LevelTimer>600*Multiplier
		If Language=0
			CenterText("GET THE FULL VERSION",17,1,63)	
			CenterText("THANKS, MAYBE LATER",18,1,64)
		Else If Language=1
			CenterText("VOLLVERSION KAUFEN",17,1,63)	
			CenterText("DANKE, NICHT JETZT",18,1,64)
		Else If Language=2
			CenterText("JE VEUX LA VERSION COMPLETE",17,1,63)	
			CenterText("MERCI, PEUT-ETRE PLUS TARD",18,1,64)

		EndIf
		PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(17+CurrentMenuOption)*.044+.027,1.1
		ScaleEntity MenuSelector,25*.012,.02,0.001
	Else
		CenterText("",17,1,63)
		PositionEntity MenuSelector,-.5+24.4*.02+.01,0.372-Float(25+CurrentMenuOption)*.044+.027,1.1
		ScaleEntity MenuSelector,20*.012,.02,0.001
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
		PositionTexture MenuBackGroundTex,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
		PositionTexture MenuBackGroundTex2,Float(LevelTimer Mod 5120)/5120.0, -Float(LevelTimer Mod 1024)/1024.0
	
		
		
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	Flip

	
	If (LevelTimer>600)
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
				If CurrentMenuOption=1
					EndMenu()
					GameExit=True
				Else
					EndMenu()
					; go to site!
					VersionNo1=-1
					GameExit=True
				EndIf
				
			EndIf
		Else If KeyDown(1)
			If CursorKeyPressed=False
				CursorKeyPressed=True
				EndMenu()
				GameExit=True
			EndIf
	
		Else
			CursorKeyPressed=False
		EndIf
	EndIf


	
End Function


Function MenuLoop12()

	Repeat
		TweenElapsed=MilliSecs()-TweenTime
	Until TweenElapsed>TweenPeriod
	
	

	;how many 'frames' have elapsed	
	TweenTicks=TweenElapsed/TweenPeriod
	
	
	
	;fractional remainder
	Tween#=Float(TweenElapsed Mod TweenPeriod)/Float(TweenPeriod)
	
	
	j=11
	If Language=0
		If LevelTimer>500 And LevelTimer<1400
			CenterText("And so,",4+j,1,50)
			If LevelTimer>900 CenterText("at the end of their journey,",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>1800 And LevelTimer<2600
			CenterText("with all the Rainbow Coins",4+j,1,50)
			CenterText("safe and sound,",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>3000 And LevelTimer<4000
			CenterText("Stinky and Loof watched a new day",4+j,1,50)
			CenterText("dawn over Wonderland.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>5000 And LevelTimer<5750
			CenterText("Peace spread across the land,",4+j,1,50)
			CenterText("",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=5750 And LevelTimer<7000
			CenterText("Peace spread across the land,",4+j,1,50)
			CenterText("touching all creatures great and small.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>8000 And LevelTimer<8500
			CenterText("The Z-Bots",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=8500 And LevelTimer<9500
			CenterText("The Z-Bots",4+j,1,50)
			CenterText("promised to be less shocking.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>11000 And LevelTimer<11500
			CenterText("The confounding Coilys",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=11500 And LevelTimer<12250
			CenterText("The confounding Coilys",4+j,1,50)
			CenterText("became more cooperative.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>13500 And LevelTimer<14000
			CenterText("And cooler heads prevailed",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=14000 And LevelTimer<15000
			CenterText("And cooler heads prevailed",4+j,1,50)
			CenterText("among the Kaboom!s.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=16000 And LevelTimer<17000
			CenterText("As for Stinky and Loof...",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=17500 And LevelTimer<18000
			CenterText("With their quest complete,",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=18000 And LevelTimer<18500
			CenterText("With their quest complete,",4+j,1,50)
			CenterText("and Wonderland saved,",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=18500 And LevelTimer<19500
			CenterText("With their quest complete,",4+j,1,50)
			CenterText("and Wonderland saved,",5+j,1,51)
			CenterText("they decided to take a well-deserved vacation!",6+j,1,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
			
		Else If LevelTimer>=20500 And LevelTimer<21000
			CenterText("And so the time has come",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=21000 And LevelTimer<21500
			CenterText("And so the time has come",4+j,1,50)
			CenterText("for Stinky and Loof",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
	
		Else If LevelTimer>=21500 And LevelTimer<22000
			CenterText("And so the time has come",4+j,1,50)
			CenterText("for Stinky and Loof",5+j,1,51)
			CenterText("to take their leave of you.",6+j,1,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
				
	
		Else If LevelTimer>23000 And LevelTimer<23500
			CenterText("With heavy hearts",4+j,1,50)
			CenterText("",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>23500 And LevelTimer<24300
			CenterText("With heavy hearts",4+j,1,50)
			CenterText("filled with fond memories",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
	
		Else If LevelTimer>24800 And LevelTimer<27000
			CenterText("they bid you Good Bye,",6+j,1,50)
			CenterText("",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>28000 And LevelTimer<29000
			CenterText("",12,1,50)
			CenterText("and Good Night!",7+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>30500
			CenterText("THE END",4+j,1,50)
		Else
			;displayText(Str$(LevelTimer),0,0,1,50)
			;DisplayText(Str$(EntityX(Camera)),0,1,1,51)
			;DisplayText(Str$(EntityY(Camera)),0,2,1,52)
			;displayText(Str$(EntityZ(Camera)),0,3,1,53)
			CenterText("",0,0,50)
			CenterText("",0,0,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		EndIf
	Else If Language=1
		If LevelTimer>500 And LevelTimer<1400
			CenterText("Und so",4+j,1,50)
			If LevelTimer>900 CenterText("sehen Stinky und Loof",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>1800 And LevelTimer<2600
			CenterText("am Ende ihrer Reise",4+j,1,50)
			CenterText("",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>3000 And LevelTimer<4000
			CenterText("einem neuen Tag",4+j,1,50)
			CenterText("im Wonderland entgegen.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>5000 And LevelTimer<5750
			CenterText("Frieden breitet sich",4+j,1,50)
			CenterText("",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=5750 And LevelTimer<7000
			CenterText("Frieden breitet sich",4+j,1,50)
			CenterText("im Wonderland aus.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>8000 And LevelTimer<8500
			CenterText("Die Z-Bots",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=8500 And LevelTimer<9500
			CenterText("Die Z-Bots",4+j,1,50)
			CenterText("versprachen freundlicher zu sein.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>11000 And LevelTimer<11500
			CenterText("Die Coilys",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=11500 And LevelTimer<12250
			CenterText("Die Coilys",4+j,1,50)
			CenterText("wurden ruhiger.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>13500 And LevelTimer<14000
			CenterText("Und die Kaboom's",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=14000 And LevelTimer<15000
			CenterText("Und die Kaboom's",4+j,1,50)
			CenterText("behielten kuehlere Koepfe.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=16000 And LevelTimer<17000
			CenterText("Und Stinky und Loof?",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=17500 And LevelTimer<18000
			CenterText("Da ihre Reise erfolgreich war",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=18000 And LevelTimer<18500
			CenterText("Da ihre Reise erfolgreich war",4+j,1,50)
			CenterText("und Wonderland gerettet ist",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=18500 And LevelTimer<19500
			CenterText("Da ihre Reise erfolgreich war",4+j,1,50)
			CenterText("und Wonderland gerettet ist",5+j,1,51)
			CenterText("haben beide sich einen Urlaub verdient!",6+j,1,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
			
		Else If LevelTimer>=20500 And LevelTimer<21000
			CenterText("So ist es an der Zeit",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=21000 And LevelTimer<21500
			CenterText("So ist es an der Zeit",4+j,1,50)
			CenterText("dir lebe wohl zu sagen.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
	
		Else If LevelTimer>=21500 And LevelTimer<22000
			CenterText("So ist es an der Zeit",4+j,1,50)
			CenterText("dir lebe wohl zu sagen.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)

				
	
		Else If LevelTimer>23000 And LevelTimer<23500
			CenterText("Schweren Herzens",4+j,1,50)
			CenterText("",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>23500 And LevelTimer<24300
			CenterText("Schweren Herzens",4+j,1,50)
			CenterText("sagen Stinky und Loof",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
	
		Else If LevelTimer>24800 And LevelTimer<27000
			CenterText("auf Wiedersehn...",6+j,1,50)
			CenterText("",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>28000 And LevelTimer<29000
			CenterText("",12,1,50)
			CenterText("und Gute Nacht!",7+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>30500
			CenterText("ENDE",4+j,1,50)
		Else
			;displayText(Str$(LevelTimer),0,0,1,50)
			;DisplayText(Str$(EntityX(Camera)),0,1,1,51)
			;DisplayText(Str$(EntityY(Camera)),0,2,1,52)
			;displayText(Str$(EntityZ(Camera)),0,3,1,53)
			CenterText("",0,0,50)
			CenterText("",0,0,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		EndIf
	Else If Language=2
		If LevelTimer>500 And LevelTimer<1400
			CenterText("Alors,",4+j,1,50)
			If LevelTimer>900 CenterText("a la fin de leur periple,",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>1800 And LevelTimer<2600
			CenterText("avec un tas de sous arc-en-ciel",4+j,1,50)
			CenterText("plein les poches,",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>3000 And LevelTimer<4000
			CenterText("Stinky et Loof contemplerent un autre",4+j,1,50)
			CenterText("coucher de soleil sur Wonderland.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>5000 And LevelTimer<5750
			CenterText("La paix se repandit dans le pays,",4+j,1,50)
			CenterText("",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=5750 And LevelTimer<7000
			CenterText("La paix se repandit dans le pays,",4+j,1,50)
			CenterText("touchant toutes les creatures, grandes et petites.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>8000 And LevelTimer<8500
			CenterText("Les Zi-Bots",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=8500 And LevelTimer<9500
			CenterText("Les Zi-Bots",4+j,1,50)
			CenterText("promirent d'etre moins tannants.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>11000 And LevelTimer<11500
			CenterText("Les deconcertants Coilys",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=11500 And LevelTimer<12250
			CenterText("Les deconcertants Coilys",4+j,1,50)
			CenterText("devinrent plus cooperatifs.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>13500 And LevelTimer<14000
			CenterText("Et la plupart des Kaboom!s",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=14000 And LevelTimer<15000
			CenterText("Et la plupart des Kaboom!s",4+j,1,50)
			CenterText("se calmerent le pompon.",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=16000 And LevelTimer<17000
			CenterText("Stinky et Loof aussi...",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=17500 And LevelTimer<18000
			CenterText("leur quete achevee,",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=18000 And LevelTimer<18500
			CenterText("leur quete achevee,",4+j,1,50)
			CenterText("et Wonderland libere,",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=18500 And LevelTimer<19500
			CenterText("leur quete achevee,",4+j,1,50)
			CenterText("et Wonderland libere,",5+j,1,51)
			CenterText("ils deciderent de prendre",6+j,1,52)
			CenterText("des vacances bien merites!",7+j,1,53)
			CenterText("",0,0,54)
			
		Else If LevelTimer>=20500 And LevelTimer<21000
			CenterText("Ainsi donc, le temps est venu",4+j,1,50)
			CenterText("",5,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>=21000 And LevelTimer<21500
			CenterText("Ainsi donc, le temps est venu",4+j,1,50)
			CenterText("pour Stinky et Loof",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
	
		Else If LevelTimer>=21500 And LevelTimer<22000
			CenterText("Ainsi donc, le temps est venu",4+j,1,50)
			CenterText("pour Stinky et Loof",5+j,1,51)
			CenterText("de vous quitter.",6+j,1,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
				
	
		Else If LevelTimer>23000 And LevelTimer<23500
			CenterText("Avec le coeur gros,",4+j,1,50)
			CenterText("",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>23500 And LevelTimer<24300
			CenterText("Avec le coeur gros,",4+j,1,50)
			CenterText("remplis de precieux souvenirs",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
	
		Else If LevelTimer>24800 And LevelTimer<27000
			CenterText("ils vous disent Au Revoir,",6+j,1,50)
			CenterText("",5+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>28000 And LevelTimer<29000
			CenterText("",12,1,50)
			CenterText("et Bonne Nuit.",7+j,1,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		Else If LevelTimer>30500
			CenterText("Fin",4+j,1,50)
		Else
			;displayText(Str$(LevelTimer),0,0,1,50)
			;DisplayText(Str$(EntityX(Camera)),0,1,1,51)
			;DisplayText(Str$(EntityY(Camera)),0,2,1,52)
			;displayText(Str$(EntityZ(Camera)),0,3,1,53)
			CenterText("",0,0,50)
			CenterText("",0,0,51)
			CenterText("",0,0,52)
			CenterText("",0,0,53)
			CenterText("",0,0,54)
		EndIf



	
	EndIf

		
	
	
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

		
		; do the game in here
		
		If LevelTimer<10000
			; fade in
			LightColor Levellight,-255+510*LevelTimer/10000,-255+510*LevelTimer/10000,-255+510*LevelTimer/10000
			AmbientLight 255*LevelTimer/10000,255*LevelTimer/10000,255*LevelTimer/10000
		EndIf
		If LevelTimer<6000
			PositionEntity Camera,20.5,-23+30*Sin(Leveltimer/66.6666),-102+6*Sin(LevelTimer/66.6666)
		EndIf
		If LevelTimer>3000 And LevelTimer<=8800
			RotateEntity Camera,10-10*Cos((LevelTimer-3000.0)/16.6666),0,0
		EndIf
		If LevelTimer>6000 And leveltimer<10680 
			PositionEntity Camera,13.5+7.0*Cos((LevelTimer-6000.0)/26.0),3+4*Cos((LevelTimer-6000.0)/21),-50-46.0*Cos((LevelTimer-6000.0)/100.0)
		EndIf
		If LevelTimer>=10680 And leveltimer<13560 
			PositionEntity Camera,9.5+3.0*Cos((LevelTimer-6000.0)/26.0),3+4*Cos((LevelTimer-6000.0)/21),-50-46.0*Cos((LevelTimer-6000.0)/100.0)
		EndIf

		If LevelTimer>=13560 And leveltimer<17340 
			PositionEntity Camera,9.5+3.0*Cos((LevelTimer-6000.0)/26.0),5+2*Cos((LevelTimer-6000.0)/21),-50-46.0*Cos((LevelTimer-6000.0)/100.0)
		EndIf
		If LevelTimer>=17340 And leveltimer<22000
			PositionEntity Camera,9.5+3.0*Cos((LevelTimer-6000.0)/26.0),4-Cos((LevelTimer-17340.0)/31.0),-50-46.0*Cos((LevelTimer-6000.0)/100.0)
		EndIf
		If LevelTimer>22000 And LevelTimer<26000
			RotateEntity Camera,10-10*Cos(5800.0/16.6666),-90.0+90.0*Cos((LevelTimer-22000.0)/22.2222),0
		EndIf
		

		
		If LevelTimer>22000 And LevelTimer<30000
			If LevelTimer<22020 And ShiftKeyPressed=False
				AnimateMD2 StinkyEntity,1,0.07,44,52
				AnimateMD2 LoofEntity,1,0.1,44,52
				TurnEntity StinkyEntity,0,20,0,True
				TurnEntity LoofEntity,0,10,0,True
				ShiftKeyPressed=True

				PositionEntity LoofEntity,EntityX(StinkyEntity)+1.5,EntityY(StinkyEntity),EntityZ(StinkyEntity)
			EndIf
			PositionEntity Camera,9.5+3.0*Cos(16000.0/26.0),4-Cos((22000-17340.0)/31.0),-50-46.0*Cos((22000-6000.0)/100.0)+(LevelTimer-22000)/200.0
		EndIf
		
		If LevelTimer>28000 And LevelTimer<30000
			; fade out
			LightColor Levellight,255-510*(LevelTimer-28000)/2000,255-510*(LevelTimer-28000)/2000,255-510*(LevelTimer-28000)/2000
			AmbientLight 255-255*(LevelTimer-28000)/2000,255-255*(LevelTimer-28000)/2000,255-255*(LevelTimer-28000)/2000
			CameraClsColor Camera,170-170*(LevelTimer-28000)/2000,170-170*(LevelTimer-28000)/2000,255-255*(LevelTimer-28000)/2000
			CameraFogColor Camera,170-170*(LevelTimer-28000)/2000,170-170*(LevelTimer-28000)/2000,255-255*(LevelTimer-28000)/2000
		EndIf

		If LevelTimer>30000
			PositionEntity Camera,9.5+3.0*Cos(20000.0/26.0),4-Cos((22000-17340)/31.0),100
		EndIf
		

		
		ControlMonsters()
		; Items
		ControlItems()
		; Fish
		ControlFish()
		; Particles
		ControlParticles()
		For i=0 To NofMonsters-1
			If MonsterMoving(i)=True
				TranslateEntity MonsterEntity(i),MonsterMovingX(i)/Float(MonsterSpeed(i)),0,-MonsterMovingY(i)/Float(MonsterSpeed(i))
				TranslateEntity MonsterShadowEntity(i),MonsterMovingX(i)/Float(MonsterSpeed(i)),0,-MonsterMovingY(i)/Float(MonsterSpeed(i))
				MonsterMovingTimer(i)=MonsterMovingTimer(i)-1
				
			EndIf
		Next
		For i=0 To NofMonsters-1
			If MonsterMoving(i)=True And MonsterMovingTimer(i)=0
				EndMoveMonster(i)
			EndIf
		Next
		CleanUpMonsters()
		CleanUpItems()
		PositionTexture RainbowTexture,Watertexu,WaterTexv*10


	


		
		
		
		
		
		
		
		LevelTimer=LevelTimer+1
		UpdateWorld
	
	Next
		
	RenderWorld tween
	
	
	

	
	
	
	
	
	
	
	Flip
	If LevelTimer<12000
		For i=0 To 50
			PositionEntity FireBallEntity(i),20.0-22.0*Sin((LevelTimer-60.0*i-7000)/15)+1.0*Sin(i*35),7.0*Cos((LevelTimer-60.0*i-7000)/5)-(LevelTimer-60.0*i-7000.0)/200.0+1.0*Sin(i*57),-95+(LevelTimer-60.0*i-7000.0)/30.0+1.0*Cos(i*109)
			RotateEntity FireBallEntity(i),0,0,(LevelTimer+10*i) Mod 360
		Next
	Else
		For i=0 To 50
			If LevelTimer<22000
				If LevelTimer>18000
					ScaleEntity FireBallentity(i),(LevelTimer-18000.0)/1000.0,(LevelTimer-18000.0)/1000.0,0.001
				Else
					ScaleEntity FireBallEntity(i),0,0,0.001
				EndIf
			EndIf
			;If Sin((LevelTimer-60.0*i-7000)/15)>0
				PositionEntity FireBallEntity(i),34.0+90.0*Cos((LevelTimer-60.0*i-7000)/8.3333),-10+(30.0*Sin((LevelTimer-60.0*i-7000)/4.1666)),500
			;Else
			;	PositionEntity FireBallEntity(i),14.0-140.0*Cos((LevelTimer-60.0*i-7000)/15),-30+Abs(80.0*Sin((LevelTimer-60.0*i-7000)/15)),500
			;EndIf
			RotateEntity FireBallEntity(i),0,0,(LevelTimer+10*i) Mod 360
		Next
	EndIf
	
	
	If KeyHit(1) 
		If CursorKeyPressed=False
			EndLevel()
			StartMenu(1)
			CursorKeyPressed=True
		EndIf
	Else
		CursorKeyPressed=False
	End If


	
	
	

End Function


Function EndMenu()

	HideEntity MenuBackGround
	HideEntity MenuBackGround2
	HideEntity Logo1
	HideEntity Logo2
	HideEntity MenuSelector
	
	; Fonts
	For l.LetterType=Each LetterType
		FreeEntity l\Entity
		Delete l
	Next



End Function
Function StartLevelMusic()

	StopChannel Music
	
	If CurrentWorld+1=4 Or CurrentWorld+1=9
		Music=PlayMusic("res6.dat")
	Else If (CurrentWorld+1=2 And Version>=10) Or CurrentWorld+1=6 Or CurrentWorld+1=10 
		Music=PlayMusic("res5.dat")
	Else If CurrentWorld+1=8 
		Music=PlayMusic("res4.dat")
	Else 
		Music=PlayMusic("res3.dat")
	EndIf
	
	If CurrentWorld+1=8
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
	
		
	; and create new player file on disk
	If CurrentPlayerName$="" Then CurrentPlayerName$="ANONYMOUS"
	
	File=ReadFile("players/"+Lower$(CurrentPlayerName$)+".pla")

	If File=0
		CloseFile(File)
	Else
		CloseFile(File)
		CopyFile "players/"+Lower$(CurrentPlayerName$)+".pla","players/"+Lower$(CurrentPlayerName$)+".bak"
	EndIf

	
	File=WriteFile("players/"+Lower$(CurrentPlayerName$)+".pla")
	
	;*********************** LATER:

	; Need error handling if file dne
	
	;*****************************

	j=0
	WriteString(File,"Stinky & Loof Player File")	
	WriteString(File,CurrentPlayerName$)
	WriteInt (File,CurrentWorld)
	WriteInt (File,j)		; # of levels completed
	WriteInt (File,j)		; total score
	For i=0 To PlayGroundLevel-1
		WriteInt(File,CurrentPlayerLevelScore(i))
	Next

	CloseFile(File)
	
	CurrentLevel=CurrentWorld*10
	
	

End Function
Function LoadPlayer(FileName$)
	File=ReadFile("players/"+Lower$(FileName$)+".pla")
	
	;*********************** LATER:

	; Need error handling if file dne
	
	;*****************************

	CurrentPlayerName$=ReadString$(File)
	CurrentPlayerName$=ReadString$(File)
	CurrentWorld= ReadInt(File)
	j=ReadInt(File)		; # of levels completed - don't need these at this stage
	j=ReadInt(File)		; total score
	For i=0 To PlayGroundLevel-1
		CurrentPlayerLevelScore(i)=ReadInt(File)
	Next

	CloseFile(File)
	
	CurrentLevel=CurrentWorld*10
	
	
	
End Function

Function SavePlayer()
	; saves teh current player
	
	File=WriteFile("players/"+Lower$(CurrentPlayerName$)+".pla")
	
	If File=0
		Return
	EndIf

	
	
	j=0
	j2=0
	WriteString(File,"Stinky & Loof Player File")	
	WriteString(File,CurrentPlayerName$)
	WriteInt (File,CurrentWorld)
	
	For i=0 To PlayGroundLevel-1
		If CurrentPlayerLevelScore(i)>0
			j=j+1
			j2=j2+CurrentPlayerLevelScore(i)
		EndIf
	Next
				
	WriteInt (File,j)		; # of levels completed
	WriteInt (File,j2)		; total score
	For i=0 To PlayGroundLevel-1
		WriteInt(File,CurrentPlayerLevelScore(i))
	Next

	CloseFile(File)
	
	
	
End Function

Function LoadOptions()
	File=ReadFile("options.cfg")
	
	If File=0
		OptionsSoundLevel=1
		OptionsMusicLevel=1
		DetailLevel=5
		MyGfxMode=1
		MyColMode=1
		MyWindowed=0
		Language=0
		NoTimeLimit=0
		LockWorlds=0
		
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
	

	CloseFile(File)
	
	File=ReadFile("extra.cfg")
	If File=0
		IsExtraDemoLevelActive=0
		Return
	EndIf
	IsExtraDemoLevelActive=ReadInt(File)
	If IsExtraDemoLevelActive=1
		WorldLevelName$(15)="resdb1.dat"
		WorldLevelName$(16)="resdb2.dat"
	EndIf

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
	
	
	CloseFile(File)
	
	File=WriteFile("extra.cfg")
	WriteInt File,IsExtraDemoLevelActive
	CloseFile (File)
	
	
	
	
	
End Function