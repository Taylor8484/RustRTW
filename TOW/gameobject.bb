; -------------------------------------------
; Stinky
; -------------------------------------------

Function CreateStinky(x,y)
	; input tile position x/y
	
	StinkyEntity=CopyEntity(StinkyMesh) 
	PositionEntity StinkyEntity,x+0.5,0,-(y+0.5)	
	PositionEntity Camera,x+0.5,10.4+10,-y-9.5+10
	
	RotateEntity StinkyEntity,0,180,0
	StinkyRotation=180
	StinkyRotationGoal=180
	
	StinkyShadowEntity=CreateFlatSquare(RoundShadowTexture)
	ScaleEntity StinkyShadowEntity,.65,1,.65
	PositionEntity StinkyShadowEntity,x+.2,0.01,-(y+.2)
	
	StinkyX=x
	StinkyY=y
	StinkyX2=x
	StinkyY2=y
	StinkyMoving=False
	
	StinkyAnimCurrent=0
	StinkyAnimNew=0
	
	
	StinkyMoving=False
	StinkyDead=False
	StinkyDying=False
	StinkyDyingTimer=0	
	StinkyIdleCounter=0 
	
	StinkyBlinkCounter=300
	StinkyBeforeBlinkTexture=StinkyTexture
	
	;StinkyWarpSphere=CreateSphere()
	;EntityTexture StinkyWarpSphere,LavaTexture
	;PositionEntity StinkyWarpSphere,x+0.5,0.5,-(y+0.5)
	
	FocusOnStinky=True
	FocusSwitchTimer=0
	BothCompleted=0
	
End Function

Function CanStinkyMove(dx,dy)

	If StinkyDying=True Then Return True
	
	; tests a field to see if stinky can go there
	x=StinkyX+dx
	y=StinkyY+dy
	
	; First check tiles
	
	Tile=MapTile(x,y)
	
	If (Tile/100=3 Or Tile/100=9) And StinkyMoving=True ; moving into water/lava (such as from ice)
		KillStinky()
		Return False
	EndIf
	

	
	If Tile/100=0 Or Tile/100=2 Or Tile/100=3 Or Tile/100=8 Or Tile/100=9	Or (Tile/100=10 And Tile Mod 100<50)
		; Walls, Water, Cannons, Lava, Gates - Definitely Cannot Enter
		Return False
	EndIf
	
	; check loof
	If LoofExists=True
		If BothCompleted=0 And ((LoofX=x And LoofY=y) Or (LoofX2=x And LoofY2=y))
			Return False
		EndIf
	EndIf
	
	; Finally check boxes
	For i=0 To NofBoxes-1
		If (BoxX(i)=x And BoxY(i)=y) Or (BoxX2(i)=x And BoxY2(i)=y)
			; yep found a box
			; is it moving
			If BoxMoving(i)=True
				; box moving away from stinky: no problem [LATER: or sideways at end of movement]
				If BoxMovingX(i)=dx And BoxMovingY(i)=dy 
					StinkyAnimNew=1
					Return True
				EndIf				
				; box moving towards stinky: boom!
				If BoxMovingX(i)=-dx And BoxMovingY(i)=-dy
					KillStinky()
					Return False
				EndIf
				; box moving sideways: just bounce stinky off
			
				Return False
			EndIf
			; not a moving box: check if the box can be moved further
			If MapTile(BoxX(i),BoxY(i))/100<>5 Or ConveyorActive=False	; don't even check if box on moving conveyor
				If CanBoxMove(i,dx,dy)=True 
					; move it
					; check if stinky on ice, if so move box faster
					If MapTile(StinkyX,StinkyY)/100=4 Or (MapTile(StinkyX,StinkyY)/100=5 And ConveyorActive=True)
						MoveBox(i,dx,dy,2)
						
					Else
						MoveBox(i,dx,dy,0)
					EndIf
					; and move stinky
					StinkyAnimNew=2
					Return True
				Else 
					; box can't be moved, so neither can stinky
					Return False
				EndIf
			Else 
				Return False
			EndIf
		EndIf
	Next
	
	;Default: Move normally
	If (MapTile(StinkyX,StinkyY)/100=5 And ConveyorActive=True) Or MapTile(StinkyX,StinkyY)/100=4
		StinkyAnimNew=0
	Else
		StinkyAnimNew=1
	EndIf
	Return True
		
End Function

Function MoveStinky(dx,dy)
	StinkyIdleCounter=0
	EntityTexture StinkyEntity,StinkyTexture
	If MD2Animating(StinkyEntity)=False Or StinkyAnimCurrent<>StinkyAnimNew
		If StinkyAnimNew=0 ; nothing
			AnimateMD2 StinkyEntity,0,1,0,0
		Else If StinkyAnimNew=1 ; normal walk
			AnimateMD2 StinkyEntity,1,.25,2,18
		Else If StinkyAnimNew=2 ; pushing walk
			AnimateMD2 StinkyEntity,1,.25,19,35
		EndIf
		StinkyAnimCurrent=StinkyAnimNew
	EndIf
		
	If MapTile(StinkyX,StinkYy)/100=4
		;ice
		PlayMySound(SFXIce,StinkyEntity)
	EndIf	
		
	; Speed depends on Terrain
	If MapTile(StinkyX,StinkyY)/100=4 Or (MapTile(StinkyX,StinkyY)/100=5 And ConveyorActive=True) ; Ice or Conveyor
		StinkySpeed=BaseSpeed/2
	Else
		StinkySpeed=BaseSpeed
	EndIf
	
		

	StinkyMovingTimer=StinkySpeed
	StinkyX2=StinkyX+dx
	StinkyY2=StinkyY+dy
	
	StinkyMoving=True
	StinkyMovingX=dx
	StinkyMovingY=dy
	
	If dx=0
		If dy=1	
			StinkyRotationGoal=180
		Else 
			StinkyRotationGoal=0
		EndIf
	Else
		If dx=1
			StinkyRotationGoal=270
		Else 
			StinkyRotationGoal=90
		EndIf
	EndIf
		
			

End Function

Function EndMoveStinky()
	
	OldX=StinkyX
	OldY=StinkyY
	OldTile=MapTile(StinkyX,StinkyY) ;what did stinky used to stand on?
									 ; used e.g. for breakawaytiles
	StinkyX=StinkyX2
	StinkyY=StinkyY2
	
	If OldTile/100=6
		; Breakaway Tiles - look at what stinky used to stand on
		
		If OldTile=600	
			; will break away into water
			; convert logic to water
			MapTile(OldX,OldY)=300
			; and destroy that tile
			KillBreakAway(OldX,OldY)
			
		Else If OldTile=603
			; will break into lava
			MapTile(OldX,OldY)=900
			KillBreakAway(OldX,OldY)
		Else
			; reduce by one
			KillBreakAway(OldX,OldY)
			MapTile(OldX,OldY)=OldTile-1
			CreateBreakAway(OldX,OldY,OldTile-601)
		EndIf
		
	EndIf
	If OldTile/100=11
		; Button Tiles 
		StepOffButton(OldX,OldY)
	EndIf
	; Signs
	For i=0 To 4
		If OldX=SignX(i) And OldY=SignY(i)
			If SignActive=1 Then StopSign(11+i)
		EndIf
	Next 
	
		
	; now check what tile its standing on now
	Tile=MapTile(StinkyX,StinkyY)
	If Tile/100=4 
		; Ice, keep sliding if possible 
	
		;first time? Play sound
		If OldTile/100<>4 ; first time? Play sound
			i=Rand(0,4)
			If DoesSoundExist=True Then SoundPitch SFXSlIce(i),16000
			PlayMySound(SFXSlIce(i),StinkyEntity)
		EndIf

	
		; first, figure out what kind of ice (curved, straight) to find forward/back directions
		
		If Tile=400	; straight ice
			; straight forward
			ForwardX=StinkyMovingX
			ForwardY=StinkyMovingY
			BackwardX=-StinkyMovingX
			BackwardY=-StinkyMovingY
		Else If Tile=401 ; down-right ice
			If StinkyMovingX=-1 ; coming from right
				ForwardX=0
				ForwardY=1
				BackWardX=1
				BackWardY=0
			Else				; coming from below
				ForwardX=1
				ForwardY=0
				BackWardX=0
				BackWardY=1
			EndIf
		Else If Tile=402 ; down-left ice
			If StinkyMovingX=1  ; coming from left
				ForwardX=0
				ForwardY=1
				BackWardX=-1
				BackWardY=0
			Else				; coming from below
				ForwardX=-1
				ForwardY=0
				BackWardX=0
				BackWardY=1
			EndIf
		Else If Tile=403 ; up-left ice
			If StinkyMovingX=1 ; coming from left
				ForwardX=0
				ForwardY=-1
				BackWardX=-1
				BackWardY=0
			Else				; coming from above
				ForwardX=-1
				ForwardY=0
				BackWardX=0
				BackWardY=-1
			EndIf
		Else If Tile=404 ; up-right ice
			If StinkyMovingX=-1 ; coming from right
				ForwardX=0
				ForwardY=-1
				BackWardX=1
				BackWardY=0
			Else				; coming from above
				ForwardX=1
				ForwardY=0
				BackWardX=0
				BackWardY=-1
			EndIf
		EndIf
	

		; now do the check and movement
		If CanStinkyMove(ForwardX,ForwardY)=True
			MoveStinky(ForwardX,ForwardY)
			Return
		; can't move forward, can we move back?
		Else If CanStinkyMove(BackwardX,BackwardY)=True
			MoveStinky(BackwardX,BackwardY)
			Return
		Else
			; couldn't move further... this should never happen!
			KillStinky() ;but just in case
			Return
		EndIf
		
			
	EndIf
	
	If Tile/100=5 And ConveyorActive=True
		
		; Conveyor Belts: move if possible (no bouncing or anything,
		;	anything that collides with an immovable is killed)
		
		If OldTile<>Tile ; stepped onto this direction first time
			PlayMySound(SFXStinkyConvey,StinkyEntity)
		EndIf
		
		If Tile=500	; up
			If CanStinkyMove(0,-1)=True 
				MoveStinky(0,-1)
				Return
			Else
				KillStinky()
				Return
			EndIf
		EndIf
		If Tile=502	; down
			If CanStinkyMove(0,1)=True 
				MoveStinky(0,1)
				Return
			Else
				KillStinky()
				Return

			EndIf
		EndIf
		If Tile=501	; right
			If CanStinkyMove(1,0)=True 
				MoveStinky(1,0)
				Return
			Else
				KillStinky()
				Return

			EndIf
		EndIf
		If Tile=503	; left
			If CanStinkyMove(-1,0)=True 
				MoveStinky(-1,0)
				Return
			Else
				KillStinky()
				Return

			EndIf
		EndIf
	
	EndIf

	If Tile/100=11	; buttons
		StepOnButton(StinkyX,StinkyY)
	EndIf	
	
	If Tile/100=12 	; Teleporter
		StinkyMoving=False	; do first to avoid 'death in water'
		If StinkyStepOnTeleporter(StinkyX,StinkyY,StinkyMovingX,StinkyMovingY)=True
			Return
		EndIf
	EndIf
	
	; Signs
	For i=0 To 4
		If StinkyX=SignX(i) And StinkyY=SignY(i)
			If FocusOnStinky=True
				StartSign(SignText$(i),8,11+i,1)
			EndIf
		EndIf
	Next 

	
			
		
			
	
	StinkyMoving=False
	
	
End Function

Function ControlStinky()
	;animations

	; turning into walk-direction
	; note: actual turning is taken care of in boxcontrol
	If StinkyRotation<>StinkyRotationGoal
		If StinkyRotation<StinkyRotationGoal And StinkyRotationGoal-StinkyRotation<180
			StinkyRotation=StinkyRotation+5
		Else If StinkyRotation>StinkyRotationGoal And StinkyRotation-StinkyRotationGoal>180
			StinkyRotation=StinkyRotation+5
	  	Else
			StinkyRotation=StinkyRotation-5
		EndIf
		If StinkyRotation=360 Then StinkyRotation=0
		If StinkyRotation=-5 Then StinkyRotation=355
	EndIf
	
End Function

Function KillStinky()

	If StinkyDying=True Return
	If LevelCompleted>0 Return
	
	PlayMySound(SFXDeath,0)
	
	AnimateMD2 StinkyEntity,3,0.2,36,42
	EntityTexture StinkyEntity,StinkyTextureB
	StinkyAnimCurrent=3
	
	StinkyX=0
	StinkyY=0
	
	StinkyDyingTimer=100
	StinkyDying=True
	
	If FocusOnStinky=False
		SwitchStinkyLoof()
	EndIf
	
	FreeEntity StinkyShadowEntity

End Function

; -------------------------------------------
; Loof
; -------------------------------------------

Function CreateLoof(x,y)
	; input tile position x/y
	
	LoofExists=True
	
	LoofEntity=CopyEntity(LoofMesh) 
	PositionEntity LoofEntity,x+0.5,0,-(y+0.5)	
	
	RotateEntity LoofEntity,0,180,0
	LoofRotation=180
	LoofRotationGoal=180

	LoofShadowEntity=CreateFlatSquare(RoundShadowTexture)
	ScaleEntity LoofShadowEntity,.55,1,.55
	PositionEntity LoofShadowEntity,x+.25,0.01,-(y+.25)


	
	LoofX=x
	LoofY=y
	LoofX2=x
	LoofY2=y
	LoofMoving=False
	
	LoofAnimCurrent=0
	LoofAnimNew=0
	LoofIdleCounter=0
	
	
	LoofMoving=False
	LoofDead=False
	LoofDying=False
	LoofDyingTimer=0	
	
	LoofBlinkCounter=370
	LoofBeforeBlinkTexture=LoofTexture

	
	;LoofWarpSphere=CreateSphere()
	;EntityTexture LoofWarpSphere,LavaTexture
	;PositionEntity LoofWarpSphere,x+0.5,0.5,-(y+0.5)
	
End Function

Function CanLoofMove(dx,dy)

	If LoofDying=True Then Return True

	; tests a field to see if Loofcan go there
	x=LoofX+dx
	y=LoofY+dy
	
	; First check tiles
	
	Tile=MapTile(x,y)
	
	If (Tile/100=3 Or Tile/100=9) And LoofMoving=True ; moving into water/lava (such as from ice)
		KillLoof()
		Return False
	EndIf
	

	
	If Tile/100=0 Or Tile/100=2 Or Tile/100=3 Or Tile/100=8 Or Tile/100=9	Or (Tile/100=10 And Tile Mod 100<50)
		; Walls, Water, Cannons, Lava, Gates - Definitely Cannot Enter
		Return False
	EndIf
	
	; check stinky
	If BothCompleted=0 And ((StinkyX=x And StinkyY=y) Or (StinkyX2=x And StinkyY2=y))
		Return False
	EndIf	
	
	; Finally check boxes
	For i=0 To NofBoxes-1
		If (BoxX(i)=x And BoxY(i)=y) Or (BoxX2(i)=x And BoxY2(i)=y)
			; yep found a box
			; is it moving
			If BoxMoving(i)=True
				; box moving away from Loof: no problem [LATER: or sideways at end of movement]
				If BoxMovingX(i)=dx And BoxMovingY(i)=dy 
					LoofAnimNew=1
					Return True
				EndIf				
				; box moving towards Loof: boom!
				If BoxMovingX(i)=-dx And BoxMovingY(i)=-dy
					KillLoof()
					Return False
				EndIf
				; box moving sideways: just bounce Loofoff
			
				Return False
			EndIf
			; not a moving box: check if the box can be moved further
			If MapTile(BoxX(i),BoxY(i))/100<>5 Or ConveyorActive=False	; don't even check if box on moving conveyor
				If CanBoxMove(i,dx,dy)=True 
					; move it
					; check if Loofon ice, if so move box faster
					If MapTile(LoofX,LoofY)/100=4 Or (MapTile(LoofX,LoofY)/100=5 And ConveyorActive=True)
						MoveBox(i,dx,dy,2)
						
					Else
						MoveBox(i,dx,dy,0)
					EndIf
					; and move Loof
					LoofAnimNew=2
					Return True
				Else 
					; box can't be moved, so neither can Loof
					Return False
				EndIf
			Else 
				Return False
			EndIf
		EndIf
	Next
	
	;Default: Move	
	If (MapTile(LoofX,LoofY)/100=5 And ConveyorActive=True) Or MapTile(LoofX,LoofY)/100=4
		LoofAnimNew=0
	Else
		LoofAnimNew=1
	EndIf
	Return True
		
End Function

Function MoveLoof(dx,dy)
	LoofIdleCounter=0
	EntityTexture LoofEntity,LoofTexture
	If MD2Animating(LoofEntity)=False Or LoofAnimCurrent<>LoofAnimNew
		If LoofAnimNew=0 ; nothing
			AnimateMD2 LoofEntity,0,1,0,0
		Else If LoofAnimNew=1 ; normal walk
			AnimateMD2 LoofEntity,1,.25,2,18
		Else If LoofAnimNew=2 ; pushing walk
			AnimateMD2 LoofEntity,1,.25,19,35
		EndIf
		LoofAnimCurrent=LoofAnimNew
	EndIf

	If MapTile(LoofX,Loofy)/100=4
		;ice
		PlayMySound(SFXIce,LoofEntity)
	EndIf	
		
	; Speed depends on Terrain
	If MapTile(LoofX,LoofY)/100=4 Or (MapTile(LoofX,LoofY)/100=5 And ConveyorActive=True) ; Ice or Conveyor
		LoofSpeed=BaseSpeed/2
	Else
		LoofSpeed=BaseSpeed
	EndIf
	
	

	LoofMovingTimer=LoofSpeed
	LoofX2=LoofX+dx
	LoofY2=LoofY+dy
	
	LoofMoving=True
	LoofMovingX=dx
	LoofMovingY=dy
	
	If dx=0
		If dy=1	
			LoofRotationGoal=180
		Else 
			LoofRotationGoal=0
		EndIf
	Else
		If dx=1
			LoofRotationGoal=270
		Else 
			LoofRotationGoal=90
		EndIf
	EndIf


End Function

Function EndMoveLoof()
	
	OldX=LoofX
	OldY=LoofY
	OldTile=MapTile(LoofX,LoofY) ;what did Loofused to stand on?
									 ; used e.g. for breakawaytiles
	LoofX=LoofX2
	LoofY=LoofY2
	
	If OldTile/100=6
		; Breakaway Tiles - look at what Loofused to stand on
		
		If OldTile=600	
			; will break away into water
			; convert logic to water
			MapTile(OldX,OldY)=300
			; and destroy that tile
			KillBreakAway(OldX,OldY)
		Else If OldTile=603
			; will break into lava
			MapTile(OldX,OldY)=900
			KillBreakAway(OldX,OldY)
		Else
			; reduce by one
			KillBreakAway(OldX,OldY)
			MapTile(OldX,OldY)=OldTile-1
			CreateBreakAway(OldX,OldY,OldTile-601)
		EndIf
		
	EndIf
	If OldTile/100=11
		; Button Tiles 
		StepOffButton(OldX,OldY)
	EndIf
	; Signs
	For i=0 To 4
		If OldX=SignX(i) And OldY=SignY(i)
			If SignActive=2 Then StopSign(11+i)
		EndIf
	Next 

	
		
	; now check what tile its standing on now
	Tile=MapTile(LoofX,LoofY)
	If Tile/100=4 
		
		; Ice, keep sliding if possible 
		
		If OldTile/100<>4 ; first time? Play sound
			i=Rand(0,4)
			If DoesSoundExist=True Then SoundPitch SFXSLIce(i),19000
			PlayMySound(SFXSlIce(i),LoofEntity)
		EndIf
		
		
		; first, figure out what kind of ice (curved, straight) to find forward/back directions
		
		If Tile=400	; straight ice
			; straight forward
			ForwardX=LoofMovingX
			ForwardY=LoofMovingY
			BackwardX=-LoofMovingX
			BackwardY=-LoofMovingY
		Else If Tile=401 ; down-right ice
			If LoofMovingX=-1 ; coming from right
				ForwardX=0
				ForwardY=1
				BackWardX=1
				BackWardY=0
			Else				; coming from below
				ForwardX=1
				ForwardY=0
				BackWardX=0
				BackWardY=1
			EndIf
		Else If Tile=402 ; down-left ice
			If LoofMovingX=1  ; coming from left
				ForwardX=0
				ForwardY=1
				BackWardX=-1
				BackWardY=0
			Else				; coming from below
				ForwardX=-1
				ForwardY=0
				BackWardX=0
				BackWardY=1
			EndIf
		Else If Tile=403 ; up-left ice
			If LoofMovingX=1 ; coming from left
				ForwardX=0
				ForwardY=-1
				BackWardX=-1
				BackWardY=0
			Else				; coming from above
				ForwardX=-1
				ForwardY=0
				BackWardX=0
				BackWardY=-1
			EndIf
		Else If Tile=404 ; up-right ice
			If LoofMovingX=-1 ; coming from right
				ForwardX=0
				ForwardY=-1
				BackWardX=1
				BackWardY=0
			Else				; coming from above
				ForwardX=1
				ForwardY=0
				BackWardX=0
				BackWardY=-1
			EndIf
		EndIf
	

		; now do the check and movement
		If CanLoofMove(ForwardX,ForwardY)=True
			MoveLoof(ForwardX,ForwardY)
			Return
		; can't move forward, can we move back?
		Else If CanLoofMove(BackwardX,BackwardY)=True
			MoveLoof(BackwardX,BackwardY)
			Return
		Else
			; couldn't move further... this should never happen!
			KillLoof() ;but just in case
			Return
		EndIf
		
			
	EndIf
	
	If Tile/100=5 And ConveyorActive=True
		; Conveyor Belts: move if possible (no bouncing or anything,
		;	anything that collides with an immovable is killed)
		
		If Tile<>OldTile ; first time this direction
			PlayMySound(SFXLoofConvey,LoofEntity)
		EndIf
		
		If Tile=500	; up
			If CanLoofMove(0,-1)=True 
				MoveLoof(0,-1)
				Return
			Else
				KillLoof()
				Return
			EndIf
		EndIf
		If Tile=502	; down
			If CanLoofMove(0,1)=True 
				MoveLoof(0,1)
				Return
			Else
				KillLoof()
				Return

			EndIf
		EndIf
		If Tile=501	; right
			If CanLoofMove(1,0)=True 
				MoveLoof(1,0)
				Return
			Else
				KillLoof()
				Return

			EndIf
		EndIf
		If Tile=503	; left
			If CanLoofMove(-1,0)=True 
				MoveLoof(-1,0)
				Return
			Else
				KillLoof()
				Return

			EndIf
		EndIf
	
	EndIf

	If Tile/100=11	; buttons
		StepOnButton(LoofX,LoofY)
	EndIf	
	
	If Tile/100=12 	; Teleporter
		LoofMoving=False	; do first to avoid 'death in water'
		If LoofStepOnTeleporter(LoofX,LoofY,LoofMovingX,LoofMovingY)=True
			Return
		EndIf
	EndIf
	
	; Signs
	For i=0 To 4
		If LoofX=SignX(i) And LoofY=SignY(i)
			If FocusOnStinky=False Then StartSign(SignText$(i),8,11+i,1)
		EndIf
	Next 

	
			
		
			
	
	LoofMoving=False
	
	
End Function

Function ControlLoof()
	;animations

	; turning into walk-direction
	; note: actual turning is taken care of in boxcontrol
	If LoofRotation<>LoofRotationGoal
		If LoofRotation<LoofRotationGoal And LoofRotationGoal-LoofRotation<180
			LoofRotation=LoofRotation+5
		Else If LoofRotation>LoofRotationGoal And LoofRotation-LoofRotationGoal>180
			LoofRotation=LoofRotation+5
	  	Else
			LoofRotation=LoofRotation-5
		EndIf
		If LoofRotation=360 Then LoofRotation=0
		If LoofRotation=-5 Then LoofRotation=355
	EndIf
	
End Function


Function KillLoof()
	If LoofExists=False Return
	If LoofDying=True Return
	If LevelCompleted>0 Return

	
	PlayMySound(SFXDeath,0)
	
	AnimateMD2 LoofEntity,3,0.2,36,42
	EntityTexture LoofEntity,LoofTextureB
	LoofAnimCurrent=3
		
	LoofX=0
	LoofY=0
	
	LoofDyingTimer=100
	LoofDying=True
	
	If FocusOnStinky=True
		SwitchStinkyLoof()
	EndIf
	
	FreeEntity LoofShadowEntity

End Function

Function SwitchStinkyLoof()
	
	If LoofExists=False Return
	If FocusOnStinky=True And BothCompleted/2=1 Return
	If FocusOnStinky=False And BothCompleted Mod 2 = 1 Return
	If LevelTimer<160 Return	; no switching while zoom in or one already out

	PlayMySound(SFXTeleport,0)
	
	; switches Control and Camera on Stinky and Loof
	FocusOnStinky=Not FocusOnStinky
	FocusSwitchTimer=50
	; also move camera
	If FocusOnStinky=True
		CameraDX=EntityX(StinkyEntity)-EntityX(Camera)
		CameraDZ=(EntityZ(StinkyEntity)-9)-EntityZ(Camera)
	Else
		CameraDX=EntityX(LoofEntity)-EntityX(Camera)
		CameraDZ=(EntityZ(LoofEntity)-9)-EntityZ(Camera)
	EndIf
	
	; and handle signs
	For i=11 To 15
		StopSign(i)
	Next
	CheckSigns()
	
	
	


End Function

; -------------------------------------------
; Boxes
; -------------------------------------------

Function CreateBox(x,y,btype)
	; input tile position x/y
	
	If btype=0
		BoxEntity(NofBoxes)=CopyEntity(WoodBoxMesh) 
	Else If btype=1
		BoxEntity(NofBoxes)=CopyEntity(SteelBoxMesh)
	EndIf
	BoxType(NofBoxes)=btype
	
		
	PositionEntity BoxEntity(NofBoxes),x+0.5,0.4,-(y+0.5)	
	
	BoxX(NofBoxes)=x
	BoxY(NofBoxes)=y
	BoxX2(NofBoxes)=x
	BoxY2(NofBoxes)=y
	BoxMoving(NofBoxes)=False
	BoxDead(NofBoxes)=False
	
	BoxSwayingTimer(NofBoxes)=-1
	
	;BoxShadowEntity(NofBoxes)=CopyEntity(BoxShadowMesh,BoxEntity(NofBoxes))
	;ScaleEntity BoxShadowEntity(NofBoxes),2.5,1,2.5
	;TranslateEntity BoxShadowEntity(NofBoxes),-.9,-.999,.9
			
	NofBoxes=NofBoxes+1
	
	
	
	

End Function

Function CanBoxMove(i,dx,dy)
	
	; can't move if already moving
	;If BoxMoving(i)=True
	;	Return False
	;EndIf
	
	; tests a field to see if the box can go there
	x=BoxX(i)+dx
	y=BoxY(i)+dy
	
	; First check tiles
	Tile=MapTile(x,y)
	If Tile/100=0 Or Tile/100=2 Or Tile/100=6	Or Tile/100=8 Or (Tile/100=10 And Tile Mod 100<50)
		; Walls, BreakAways, Cannons, Gates - Definitely Cannot Enter
		Return False
	EndIf
	
	; Now check boxes 
	; if the box we're running into is stationary, then push that away (if possible)
	; exception: wood box can't push steel box
	; if the box is moving, kill both boxes
	For j=0 To NofBoxes-1
		If ((BoxX(j)=x And BoxY(j)=y) Or (BoxX2(j)=x And BoxY2(j)=y)) And j<>i
			; yep found a box 
			; if my box is not moving: no power, can't move
			If BoxMoving(i)=False
				Return False
			EndIf

			; is the other box stationary and our box moving
			If BoxMoving(j)=False
				
				;depends on type
				If BoxType(i)=1 And BoxType(j)=0	; Steel on Wood
					; pushes or kills wood box, and definitely moves itself
					If CanBoxMove(j,dx,dy)=True
						; yes, move other box: check if on ice or conveyor
						If MapTile(BoxX(i),BoxY(i))/100 = 4 Or (MapTile(BoxX(i),BoxY(i))/100=5 And ConveyorActive=True)
							; yes, move other box faster a
							MoveBox(j,dx,dy,2)
						Else
							MoveBox(j,dx,dy,0)
						EndIf
					Else
						; no, just kill other box and move on
						KillBox(j)
					EndIf
					Return True
				EndIf
				
				If BoxType(i)=BoxType(j) ; same type
					; pushes other box if possible, if not it can't move 
					If CanboxMove(j,dx,dy)=True
						; yes, move other box: check if on ice or conveyor
						If MapTile(BoxX(i),BoxY(i))/100 = 4 Or (MapTile(BoxX(i),BoxY(i))/100=5 And ConveyorActive=True)
							; yes, move other box faster 
							MoveBox(j,dx,dy,2)
							Return True
						Else
							MoveBox(j,dx,dy,0)
							Return True
						EndIf
					Else
						; no- can't move
						Return False
						
					EndIf
					
				EndIf
				
				If BoxType(i)=0 And BoxType(j)=1 	; wood on steel
					
					; can't move
					Return False
				EndIf
				
				
			Else	
				; both boxes moving
			
				; Box moving away [LATER: or sideways at end of movement]: no interaction
				If dx=BoxMovingX(j) And dy=BoxMovingY(j)
					Return True	; no problem
				Else
					; box moving into eachother head-on or sideways: kill both
					; unless one is a wood and one is steel
					If BoxType(i)=0 And BoxType(j)=1
						KillBox(i)
						Return False
					Else If BoxType(i)=1 And BoxType(j)=0
						KillBox(j)
						Return True ; since it can keep moving
					Else
						Killbox(i)
						Killbox(j)
						Return False
					EndIf
				EndIf
			EndIf
			; default: can't move if box is in way, but should never get here
			Return False
		EndIf
	Next
	
	; now check monsters - stationary box can't move, moving box kills monster
	For j=0 To NofMonsters-1
		If ((MonsterX(j)=x And MonsterY(j)=y) Or (MonsterX2(j)=x And MonsterY2(j)=y)) 
			; yep found a monster 
			; if my box is not moving: no power, can't move
			If BoxMoving(i)=False
				Return False
			Else 
				KillMonster(j)
				
				Return True
			EndIf
		EndIf
	Next
	
	; now check items - can't move into
	For j=0 To NofItems-1
		If ((ItemX(j)=x And ItemY(j)=y)) 
			; yep found a Item 
			Return False
		EndIf
	Next






	; default: move
	Return True
End Function

Function MoveBox(i,dx,dy,Speed)

	; move box in direction dx/dy. Speed is an override factor (x1,x2) if not zero

	; override speed?
	If Speed<>0
		; yes
		BoxSpeed(i)=BaseSpeed/Speed
		PlayMySound(SFXBoxPush,BoxEntity(i))
	Else
		;no
		; Speed depends on Terrain
		If MapTile(BoxX(i),BoxY(i))/100=4 Or (MapTile(BoxX(i),BoxY(i))/100=5 And ConveyorActive=True) ; Ice  or conveyor
			BoxSpeed(i)=BaseSpeed/2
		Else
			PlayMySound(SFXBoxPush,BoxEntity(i))
			BoxSpeed(i)=BaseSpeed
		EndIf
	EndIf
	
	BoxMovingTimer(i)=BoxSpeed(i)
	BoxX2(i)=BoxX(i)+dx
	BoxY2(i)=BoxY(i)+dy
	BoxMovingX(i)=dx
	BoxMovingY(i)=dy

	
	; Check if Stinky is in the way, if so: boom!
	If (BoxX2(i)=StinkyX And BoxY2(i)=StinkyY) Or (BoxX2(i)=StinkyX2 And BoxY2(i)=StinkyY2)
		;unless stinky is moving away, [LATER: or sideways at end of move]
		;(also discount conveyors, since stinky could be 'walking' against it,
		; perhaps later look at this more precisely, for now it should do)
		If StinkyMoving=True And StinkyMovingX=dx And StinkyMovingY=dy And MapTile(StinkyX,StinkyY)/100<>5
			; no interaction
		Else
			; boom
			KillStinky()
		EndIf
	EndIf
	; Check if Loof is in the way, if so: boom!
	If (BoxX2(i)=LoofX And BoxY2(i)=LoofY) Or (BoxX2(i)=LoofX2 And BoxY2(i)=LoofY2)
		;unless stinky is moving away, [LATER: or sideways at end of move]
		;(also discount conveyors, since stinky could be 'walking' against it,
		; perhaps later look at this more precisely, for now it should do)
		If LoofMoving=True And LoofMovingX=dx And LoofMovingY=dy And MapTile(LoofX,LoofY)/100<>5
			; no interaction
		Else
			; boom
			KillLoof()
		EndIf
	EndIf



	BoxMoving(i)=True
		
End Function

Function EndMoveBox(i)
	Oldx=BoxX(i)
	OldY=BoxY(i)
	
	
	BoxX(i)=BoxX2(i)
	BoxY(i)=BoxY2(i)
	
	; stepped off button
	If MapTile(Oldx,Oldy)/100=11
		StepOffButton(Oldx,Oldy)
	EndIf
	

		
	; now check what it's on (for some depends on what type of box)
	Tile=MapTile(BoxX(i),BoxY(i))
	
	If Tile/100=3 ;Water
		PlayMySound (SFXBoxSplosh,BoxEntity(i))
		If BoxType(i)=0 ; wood box
			; convert the water into walkable floor
			MapTile(BoxX(i),BoxY(i))=100
			
			; Create Water Ripple
			CreateParticle(1,4,1,BoxX(i)-.05,-.2,-BoxY(i)+.05,0,0,0,0,0,180)
		
			; drop box and 'remove it'
			TranslateEntity BoxEntity(i),0,-.9,0
			BoxSwayX(i)=BoxX(i)
			BoxSwayY(i)=BoxY(i)

			BoxX(i)=0
			BoxX2(i)=0
			BoxSwayingTimer(i)=0
		Else If BoxType(i)=1 ; steel box
			; does it already have a steelbox?
			If Tile=399
				; yes
				; convert the water into walkable floor
				MapTile(BoxX(i),BoxY(i))=100
				
				; Create Water Ripple
				CreateParticle(1,4,1,BoxX(i)-.01,-.2,-BoxY(i)+.01,0,0,0,0,0,180)

		
				; drop box and 'remove it'
				TranslateEntity BoxEntity(i),0,-.9,0
				BoxX(i)=0
				BoxX2(i)=0
			Else
				; no
				MapTile(BoxX(i),BoxY(i))=399
				
				; Create Water Ripple
				CreateParticle(1,4,1,BoxX(i)-.01,-.2,-BoxY(i)+.01,0,0,0,0,0,180)

				; drop box two levels and 'remove it'
				TranslateEntity BoxEntity(i),0,-1.9,0
				BoxX(i)=0
				BoxX2(i)=0
			EndIf
		EndIf
	EndIf
	If Tile/100=4 ; Ice
		; Ice, keep sliding if possible
		; first, figure out forward/backward direction for straight or curved ice
		If Tile=400	; straight ice
			; straight forward
			ForwardX=BoxMovingX(i)
			ForwardY=BoxMovingY(i)
			BackwardX=-BoxMovingX(i)
			BackwardY=-BoxMovingY(i)
		Else If Tile=401 ; down-right ice
			If BoxMovingX(i)=-1 ; coming from right
				ForwardX=0
				ForwardY=1
				BackWardX=1
				BackWardY=0
			Else				; coming from below
				ForwardX=1
				ForwardY=0
				BackWardX=0
				BackWardY=1
			EndIf
		Else If Tile=402 ; down-left ice
			If BoxMovingX(i)=1  ; coming from left
				ForwardX=0
				ForwardY=1
				BackWardX=-1
				BackWardY=0
			Else				; coming from below
				ForwardX=-1
				ForwardY=0
				BackWardX=0
				BackWardY=1
			EndIf
		Else If Tile=403 ; up-left ice
			If BoxMovingX(i)=1 ; coming from left
				ForwardX=0
				ForwardY=-1
				BackWardX=-1
				BackWardY=0
			Else				; coming from above
				ForwardX=-1
				ForwardY=0
				BackWardX=0
				BackWardY=-1
			EndIf
		Else If Tile=404 ; up-right ice
			If BoxMovingX(i)=-1 ; coming from right
				ForwardX=0
				ForwardY=-1
				BackWardX=1
				BackWardY=0
			Else				; coming from above
				ForwardX=1
				ForwardY=0
				BackWardX=0
				BackWardY=-1
			EndIf
		EndIf
		
		If CanBoxMove(i,ForwardX,ForwardY)=True
			MoveBox(i,ForwardX,ForwardY,0)
			Return
		; can't move forward, can we move back?
		Else If CanBoxMove(i,BackWardX,BackWardY)=True
			MoveBox(i,BackWardX,BackWardY,0)
			Return
		Else
			; couldn't move further... should never happen, but kill just in case
			KillBox(i)
		EndIf
	EndIf

	If Tile/100=5 And ConveyorActive=True		
		; Conveyor Belts: move if possible (no bouncing or anything)
		
			
		
		; none of that happened, just do conveyor as usual
		If Tile=500	; up
			If CanBoxMove(i,0,-1)=True 
				MoveBox(i,0,-1,0)
				Return
			Else
				KillBox(i)
				Return
			EndIf
		EndIf
		If Tile=502	; down
			If CanBoxMove(i,0,1)=True 
				MoveBox(i,0,1,0)
				Return
			Else
				KillBox(i)

				Return
			EndIf
		EndIf
		If Tile=501	; right
			If CanBoxMove(i,1,0)=True 
				MoveBox(i,1,0,0)
				Return
			Else
				KillBox(i)

				Return

			EndIf
		EndIf
		If Tile=503	; left
			If CanBoxMove(i,-1,0)=True 
				MoveBox(i,-1,0,0)
				Return
			Else
				KillBox(i)

				Return

			EndIf
		EndIf
	EndIf
	
	If Tile/100=9 ;Lava
		If BoxType(i)=0 ; wood box
			; destroy
			KillBox(i)
		Else If BoxType(i)=1 ; steel box
			PlayMySound(SFXLava,BoxEntity(i))
			; does it already have a steelbox?
			If Tile=999
				; yes
				; convert the water into walkable floor
				MapTile(BoxX(i),BoxY(i))=100
		
				; drop box and 'remove it'
				TranslateEntity BoxEntity(i),0,-.9,0
				BoxX(i)=0
				BoxX2(i)=0
			Else
				; no
				MapTile(BoxX(i),BoxY(i))=999
				; drop box two levels and 'remove it'
				TranslateEntity BoxEntity(i),0,-1.9,0
				BoxX(i)=0
				BoxX2(i)=0
			EndIf
		EndIf

	
	EndIf
	
	If Tile/100=11 ;buttons
		StepOnButton(BoxX(i),BoxY(i))
	EndIf

	If Tile/100=12 ;teleporter
		If BoxStepOnTeleporter(i,BoxX(i),BoxY(i),BoxMovingX(i),BoxMovingY(i))=True
			Return
		EndIf
	EndIf
	
	
	BoxMoving(i)=False

				
End Function

Function ControlBoxes()

	StinkyOn=False ; flag, is stinky/loof on a box?
	LoofOn=False
	
	; just the swaying in the water
	For i=0 To NofBoxes-1
		If BoxSwayingTimer(i)>=0
			RotateEntity BoxEntity(i),5*Cos(BoxSwayingTimer(i)),0,5*Sin(BoxSwayingTimer(i))
			BoxSwayingTimer(i)=BoxSwayingTimer(i)+1
			If BoxSwayingTimer(i)=360 Then BoxSwayingTimer(i)=0
			
			; and check if stinky is standing on it
			If StinkyDying=False And StinkyDead=False And StinkyMoving=False And StinkyX=BoxSwayX(i) And StinkyY=BoxSwayY(i)
				RotateEntity StinkyEntity,0,0,0

				TurnEntity StinkyEntity,5*Cos(BoxSwayingTimer(i)),0,5*Sin(BoxSwayingTimer(i))
				TurnEntity StinkyEntity,0,StinkyRotation,0
				StinkyOn=True
			EndIf
			
			; and check if stinky is standing on it
			If LoofDying=False And LoofDead=False And LoofMoving=False And LoofX=BoxSwayX(i) And LoofY=BoxSwayY(i)
				RotateEntity LoofEntity,0,0,0

				TurnEntity LoofEntity,5*Cos(BoxSwayingTimer(i)),0,5*Sin(BoxSwayingTimer(i))
				TurnEntity LoofEntity,0,LoofRotation,0
				LoofOn=True
			EndIf

			
			
		EndIf
	Next
	
	IceRotation=0
	If StinkyOn=False And StinkyDying=False And StinkyDead=False
		; on no box, return to normal orientation just in case
		If MapTile(StinkyX,StinkyY)/100=4 IceRotation=-15	
		RotateEntity StinkyEntity,IceRotation,StinkyRotation,0
	EndIf
	IceRotation=0
	If LoofExists=True And LoofOn=False And LoofDying=False And LoofDead=False
		; on no box, return to normal orientation just in case
		If MapTile(LoofX,LoofY)/100=4 IceRotation=-15
		RotateEntity LoofEntity,IceRotation,LoofRotation,0
	EndIf

		
	
End Function


Function KillBox(i)
	BoxDead(i)=True
	
	PlayMySound(SFXBoxExplo,BoxEntity(i))
	
	; create explosion of particles
	For j=1 To 7*DetailLevel
		CreateParticle(0,100+BoxType(i),.15,EntityX(BoxEntity(i)),EntityY(BoxEntity(i)),EntityZ(BoxEntity(i)),Rnd(-.04,.04),Rnd(.01,.06),Rnd(-.04,.04),Rnd(-10,10),.001,60*DetailLevel)
	Next
	
	; stepped off button?
	If MapTile(BoxX(i),BoxY(i))/100=11
		StepOffButton(BoxX(i),BoxY(i))
	EndIf


End Function

Function CleanUpBoxes()
	; removes any dead boxes
	For i=0 To NofBoxes-1
		If BoxDead(i)=True
			; yep, get rid of it by overwriting it with later boxes
			FreeEntity BoxEntity(i)
			
			If i<NofBoxes-1 ;unless it is the last one
				For j=i To NofBoxes-1
					BoxEntity(j)=BoxEntity(j+1)
					BoxType(j)=BoxType(j+1)
					BoxX(j)=BoxX(j+1)
					BoxY(j)=BoxY(j+1)
					BoxX2(j)=BoxX2(j+1)
					BoxY2(j)=BoxY2(j+1)
					BoxMoving(j)=BoxMoving(j+1)
					BoxMovingX(j)=BoxMovingX(j+1)
					BoxMovingY(j)=BoxMovingY(j+1)
					BoxMovingTimer(j)=BoxMovingTimer(j+1)
					BoxSpeed(j)=BoxSpeed(j+1)
					BoxDead(j)=BoxDead(j+1)
					BoxSwayingTimer(j)=BoxSwayingTimer(j+1)
					BoxSwayX(j)=BoxSwayX(j+1)
					BoxSwayY(j)=BoxSwayY(j+1)
				Next
				i=i-1

			EndIf
			NofBoxes=NofBoxes-1
		EndIf
	Next
End Function


; -------------------------------------------
; Break Away Tiles
; -------------------------------------------

Function CreateBreakAway(x,y,btype)
	; input tile position x/y
	BreakAwayEntity(NofBreakAways)=CopyEntity(BreakAwayMesh) 
	If btype Mod 3=0
		EntityTexture BreakAwayEntity(NofBreakAways),BreakAwayTexture1
	Else If btype Mod 3=1
		EntityTexture BreakAwayEntity(NofBreakAways),BreakAwayTexture2
	Else If btype Mod 3=2
		EntityTexture BreakAwayEntity(NofBreakAways),BreakAwayTexture3
	EndIf
	BreakAwayType(NofBreakAways)=btype
	
	PositionEntity BreakAwayEntity(NofBreakAways),x,0,-y	
	
	BreakAwayX(NofBreakAways)=x
	BreakAwayY(NofBreakAways)=y
		
	NofBreakAways=NofBreakAways+1
	
	
End Function

Function KillBreakAway(x,y)
	; destroys the tile mesh at position x/y
	For i=0 To NofBreakAways-1
		If BreakAwayX(i)=x And BreakAwayY(i)=y
			; found it
			PlayMySound(SFXBreakAway,BreakAwayEntity(i))
		
			; move up remaining tiles
			FreeEntity BreakAwayEntity(i)
			
			If i<NofBreakAways-1 ;unless it is the last one
				For j=i To NofBreakAways-1
					BreakAwayEntity(j)=BreakAwayEntity(j+1)
					BreakAwayType(j)=BreakAwayType(j+1)
					BreakAwayX(j)=BreakAwayX(j+1)
					BreakAwayY(j)=BreakAwayY(j+1)
					
				Next
				i=i-1

			EndIf
			NofBreakAways=NofBreakAways-1
	
			
		EndIf
	Next 
End Function

; -------------------------------------------
; Electro Tiles
; -------------------------------------------

; CreateElectro is in level.bb
Function ControlElectroTiles()
	
	For i=0 To 3
		; movement/animation of meshes

		If ElectroExists(i)=True
			; turn on/off
			ElectroTimer(i)=ElectroTimer(i)-1
			If ElectroTimer(i)=ElectroBaseTimer
				; turn on
				MoveEntity ElectroEntity(i),0,0.02,0
				;PlayElectroSound(ElectroEntity(i))
			EndIf
			If ElectroTimer(i)=-1
				; turn off
				ElectroTimer(i)=(i+2)*ElectroBaseTimer
				MoveEntity ElectroEntity(i),0,-0.02,0
			EndIf
			
			; and animate them all
			EntityTexture ElectroEntity(i),ElectroTexture,(ElectroTimer(i) Mod 16)/4
		;	EntityAlpha ElectroEntity(i),0.5
		
			; collision
			; check if _anything_ touches it if on
			If ElectroTimer(i)<=ElectroBaseTimer
				For x=0 To MapWidth-1
					For y=0 To MapHeight-1
						If ElectroMap(x,y)=i
							; Stinky?
							If StinkyMoving=True And StinkyMovingTimer<StinkySpeed/2
								If (x=StinkyX2) And (y=StinkyY2)
									KillStinky()
								EndIf
							Else
								If (x=StinkyX) And (y=StinkyY)
									KillStinky()
								EndIf
							EndIf
							; Loof?
							If (LoofExists=True)
								If LoofMoving=True And LoofMovingTimer<LoofSpeed/2
									If (x=LoofX2) And (y=LoofY2)
										KillLoof()
									EndIf
								Else
									If (x=LoofX) And (y=LoofY)
										KillLoof()
									EndIf
								EndIf
							EndIf
				
							; Boxes?
							For j=0 To NofBoxes-1
								If (x=BoxX(j)	Or x=BoxX2(j)) And (y=BoxY(j) Or y=BoxY2(j))
									KillBox(j)
								EndIf
							Next
							; Monsters?
							For j=0 To NofMonsters-1
								If (x=MonsterX(j)	Or x=MonsterX2(j)) And (y=MonsterY(j) Or y=MonsterY2(j))
									KillMonster(j)
								EndIf
							Next
							; Sound
							If ElectroTimer(i)=ElectroBaseTimer
								PlaySingleSound(x,y,0)
							EndIf

						EndIf
					Next
				Next
			EndIf	
		EndIf	
	
		
				

	Next
	
	
End Function

; -------------------------------------------
; Cannons & Fireballs
; -------------------------------------------

Function CreateCannon(x,y,speed,dx,dy)
	; input tile position x/y
	
	CannonEntity(NofCannons)=CopyEntity(CannonMesh) 
	
	CannonMovingX(NofCannons)=dx
	CannonMovingY(NofCannons)=dy
	
	CannonSpeed(NofCannons)=speed
	If CannonMovingX(NofCannons)<>0
		TurnEntity CannonEntity(NofCannons),0,0,(90*(2-CannonMovingX(NofCannons)))
	EndIf
	If CannonMovingY(NofCannons)<>0
		TurnEntity CannonEntity(NofCannons),0,0,(90*(CannonMovingY(NofCannons)+1))
	EndIf
	PositionEntity CannonEntity(NofCannons),x+.5,0,-y-.5	
	
	CannonX(NofCannons)=x
	CannonY(NofCannons)=y
	
	CannonTimer(NofCannons)=(speed+1)*CannonBaseTimer
		
	NofCannons=NofCannons+1
	
	
	
End Function

Function ControlCannons()
	For i=0 To NofCannons-1
		CannonTimer(i)=CannonTimer(i)-1
		If CannonTimer(i)=0
			;fire!
			PlaySingleSound(CannonX(i),CannonY(i),1)
			CreateFireBall(CannonX(i)+CannonMovingX(i),CannonY(i)+CannonMovingY(i),CannonMovingX(i),CannonMovingY(i))
		
			CannonTimer(i)=(CannonSpeed(i)+1)*CannonBaseTimer
		EndIf
		

	Next
End Function


Function CreateFireball(x,y,dx,dy)
	
	; input tile position x/y
	
	FireballEntity(NofFireballs)=CopyEntity(FireballMesh) 
	
	FireBallMovingX(NofFireballs)=dx
	FireBallMovingY(NofFireballs)=dy
	
	If FireballMovingX(NofFireballs)<>0
		TurnEntity FireballEntity(NofFireballs),0,-(90*(2-FireballMovingX(NofFireballs))),0	
	EndIf
	If FireballMovingY(NofFireballs)<>0
		TurnEntity FireballEntity(NofFireballs),0,-(90*(FireballMovingY(NofFireballs)+1)),0	
	EndIf

	
	PositionEntity FireballEntity(NofFireballs),x+0.5,0.5,-y-0.5	
	
	FireballX(NofFireballs)=x
	FireballY(NofFireballs)=y
	FireballX2(NofFireballs)=x+dx
	FireballY2(NofFireballs)=y+dy

	
	FireBallMovingTimer(NofFireballs)=FireBallSpeed
	FireBallDead(NofFireballs)=False
	
		
	NofFireballs=NofFireballs+1
	
	
	
End Function
Function ControlFireballs()

	For i=0 To NofFireBalls-1
	    If FireBallDead(i)=False
			
			TranslateEntity FireBallEntity(i),Float(FireBallMovingX(i))/Float(FireBallSpeed),0,-Float(FireBallMovingY(i))/Float(FireBallSpeed)
			
			If FireBallMovingTimer(i) Mod 6 =0
				CreateParticle(0,0,0.2,EntityX(FireBallEntity(i)),EntityY(FireBallEntity(i)),EntityZ(FireBallEntity(i)),0,Rnd(-.01,.01),Rnd(-.01,.01),Rnd(-1,1),0,FireBallSpeed*2)
			EndIf
			
			FireBallMovingTimer(i)=FireBallMovingTimer(i)-1
			If FireBallMovingTimer(i)=0
				;reached next tile
						
				FireBallMovingTimer(i)=FireBallSpeed
				
				FireballX(i)=FireballX(i)+FireBallMovingX(i)
				FireballY(i)=FireballY(i)+FireBallMovingY(i)
				FireballX2(i)=FireballX(i)+FireBallMovingX(i)
				FireballY2(i)=FireballY(i)+FireBallMovingY(i)
				
				
				
			EndIf
			; check for collision 
			If CanFireBallMove(i)=False
				KillFireball(i)
			EndIf
		EndIf
	
	
	Next


End Function

Function CanFireBallMove(i)
	
	; pick either first or second half of movement
	If FireBallMovingTimer(i)>=FireBallSpeed/2
		x=FireBallX(i)
		y=FireBallY(i)
	Else
		x=FireBallX2(i)
		y=FireBallY2(i)
	EndIf
	
	; check tiles 
	Tile=MapTile(x,y)
	If Tile/100=2 Or Tile/100=8 Or (Tile/100=10 And Tile Mod 100<50)
		;can't move into walls, cannons, gates
		Return False
	EndIf
	
	
	
	; check Boxes
	For i=0 To NofBoxes-1
		If BoxMoving(i)=False Or (BoxMoving(i)=True And BoxMovingTimer(i)>=BoxSpeed(i)/2)
			x2=BoxX(i)
			y2=BoxY(i)
		Else
			x2=BoxX2(i)
			y2=BoxY2(i)
		EndIf

		If x=x2 And y=y2
			If BoxType(i)=0
				KillBox(i)
			EndIf
			Return False
		EndIf
	Next 
	
	; check Monsters
	For i=0 To NofMonsters-1
		If MonsterMoving(i)=False Or (MonsterMoving(i)=True And MonsterMovingTimer(i)>=MonsterSpeed(i)/2)
			x2=MonsterX(i)
			y2=MonsterY(i)
		Else
			x2=MonsterX2(i)
			y2=MonsterY2(i)
		EndIf

		If x=x2 And y=y2
			KillMonster(i)
			Return False
		EndIf
	Next 
	
	; check Items
	For i=0 To NofItems-1
		If x=ItemX(i) And y=ItemY(i)
			Return False
		EndIf
	Next 

	; check Stinky
	If StinkyMoving=False Or (StinkyMoving=True And StinkyMovingTimer>=StinkySpeed/2)
		x2=StinkyX
		y2=StinkyY
	Else
		x2=StinkyX2
		y2=StinkyY2
	EndIf
	If x=x2 And y=y2
		KillStinky()
		Return False
	EndIf
	
	; check Loof
	If LoofExists=True
		If LoofMoving=False Or (LoofMoving=True And LoofMovingTimer>=LoofSpeed/2)
			x2=LoofX
			y2=LoofY
		Else
			x2=LoofX2
			y2=LoofY2
		EndIf
		If x=x2 And y=y2
			KillLoof()
			Return False
		EndIf
	EndIf

	

	Return True


End Function

Function KillFireball(i)

	FireballDead(i)=True
	PlaySingleSound(FireBallX(i),FireBallY(i),2)
	

End Function

Function CleanUpFireballs()
	; removes any dead Fireballs
	For i=0 To NofFireballs-1
		If FireballDead(i)=True
			; yep, get rid of it by overwriting it with later Fireballs
			FreeEntity FireballEntity(i)
			
			If i<NofFireballs-1 ;unless it is the last one
				For j=i To NofFireballs-1
					FireballEntity(j)=FireballEntity(j+1)
					FireballX(j)=FireballX(j+1)
					FireballY(j)=FireballY(j+1)
					FireballX2(j)=FireballX2(j+1)
					FireballY2(j)=FireballY2(j+1)
					FireballMovingX(j)=FireballMovingX(j+1)
					FireballMovingY(j)=FireballMovingY(j+1)
					FireballMovingTimer(j)=FireballMovingTimer(j+1)
					FireballDead(j)=FireballDead(j+1)
				Next
				i=i-1

			EndIf
			NofFireballs=NofFireballs-1
		EndIf
	Next



End Function

; -----------------------
; Laser Gates
; -----------------------
Function CreateLaserGate(x,y,col,dir)
		
	LaserGateActive(col)=True
	
	If col=0
	; iron gate - load
		LaserGateEntity(NofLaserGates)=CopyEntity(IronGateMesh)
		If dir=1 Then TurnEntity LaserGateEntity(NofLaserGates),0,0,90
	Else
	; create the mesh if laser gate
		LaserGateEntity(NofLaserGates)=CreateMesh() 
		cyl=CreateCylinder (6,False) ; an individual cylinder
		ScaleMesh cyl,0.05,0.5,0.05
		RotateMesh cyl,0,0,90
		PositionMesh cyl,0,.25,0.0
		AddMesh cyl,LaserGateEntity(NofLaserGates)
		PositionMesh cyl,0,-.375,.2165
		AddMesh cyl,LaserGateEntity(NofLaserGates)
		PositionMesh cyl,0,0,-.433
		AddMesh cyl,LaserGateEntity(NofLaserGates)
		FreeEntity cyl
		If dir=1 Then RotateEntity LaserGateEntity(NofLaserGates),0,90,0
	EndIf
	
	
	
	PositionEntity LaserGateEntity(NofLaserGates),x+.5,0.5,-y-.5
	If col=0 Then PositionEntity LaserGateEntity(NofLaserGates),x+.5,0,-y-.5


	LaserGateX(NofLaserGates)=x
	LaserGateY(NofLaserGates)=y
	LaserGateCol(NofLaserGates)=col

	;EntityTexture LaserGateEntity(NofLaserGates),LaserGateTexture(col)
	If col >0 Then EntityAlpha LaserGateEntity(NofLaserGates),0.5
	
	If col=0
		EntityColor LaserGateEntity(NofLaserGates),128,128,128
	Else If col=1
		EntityColor LaserGateEntity(NofLaserGates),255,0,255
	Else If col=2
		EntityColor LaserGateEntity(NofLaserGates),255,255,0
	Else If col=3
		EntityColor LaserGateEntity(NofLaserGates),0,255,0
	Else If col=4
		EntityColor LaserGateEntity(NofLaserGates),0,255,255
	EndIf


	
	; Create the emmiter mesh
	LaserGateEntity2(NofLaserGates)=CreateMesh()
	If col=0
		HideEntity LaserGateEntity2(NofLaserGates)
	Else
		cyl=CreateCylinder (6,True) ; an individual cylinder
		ScaleMesh cyl,0.08,0.03,0.08
		RotateMesh cyl,0,0,90
	
		PositionMesh cyl,-.47,.25,0.0
		AddMesh cyl,LaserGateEntity2(NofLaserGates)
		PositionMesh cyl,.47*2,0,0.0
		AddMesh cyl,LaserGateEntity2(NofLaserGates)
	
		PositionMesh cyl,-.47*2,-.375,.2165
		AddMesh cyl,LaserGateEntity2(NofLaserGates)
		PositionMesh cyl,.47*2,0,0
		AddMesh cyl,LaserGateEntity2(NofLaserGates)
	
		PositionMesh cyl,-.47*2,0,-.433
		AddMesh cyl,LaserGateEntity2(NofLaserGates)
		PositionMesh cyl,.47*2,0,0
		AddMesh cyl,LaserGateEntity2(NofLaserGates)
		FreeEntity cyl
		
		If dir=1 Then RotateEntity LaserGateEntity2(NofLaserGates),0,90,0
		
		PositionEntity LaserGateEntity2(NofLaserGates),x+.5,0.5,-y-.5
	
		EntityTexture LaserGateEntity2(NofLaserGates),SteelBoxTexture
	EndIf



	
	NofLaserGates=NofLaserGates+1
	
	
End Function


Function ToggleLaserGate(n,AlwaysOff)
	; goes through the map and turns on/off all gates of that type
	; if AlwaysOff=True then we always turn off the gate, never on.
	For i=0 To MapWidth-1
		For j=0 To MapHeight-1
		
			If MapTile(i,j)/100=10 And ((MapTile(i,j)-1000) Mod 50)/2=n
				; got one, toggle it:
				If MapTile(i,j) Mod 100 <50 Or AlwaysOff=True
					If MapTile(i,j) Mod 100 < 50
						MapTile(i,j)=MapTile(i,j)+50
						LaserGateActive(n)=False
						For k=0 To NofLaserGates-1
							If LaserGateX(k)=i And LaserGateY(k)=j
								If MapTile(i,j) Mod 50 <2 
									; final gate, start anim
									FinalGateAnim=60;
								Else
									; laser gate,just hide
									HideEntity LaserGateEntity(k)
								EndIf
							EndIf
						Next
					EndIf

				Else
					MapTile(i,j)=MapTile(i,j)-50
					LaserGateActive(n)=True
					For k=0 To NofLaserGates-1
						If LaserGateX(k)=i And LaserGateY(k)=j
							ShowEntity LaserGateEntity(k)
						EndIf
					Next


					; lasergate is reactivated: check and destroy anything in its path
					; Stinky
					If (StinkyX=i Or StinkyX2=i) And (StinkyY=j Or StinkyY2=j)
						KillStinky()
					EndIf
					; Loof
					If (LoofX=i Or LoofX2=i) And (LoofY=j Or LoofY2=j)
						KillLoof()
					EndIf

					; Boxes
					For k=0 To NofBoxes-1
						If (BoxX(k)=i Or BoxX2(k)=i) And (BoxY(k)=j Or BoxY2(k)=j)
							KillBox(k)
						EndIf
					Next
					; Fireballs 
					For k=0 To NofFireballs-1
						If (FireballX(k)=i Or FireballX2(k)=i) And (FireballY(k)=j Or FireballY2(k)=j)
							KillFireball(k)
						EndIf
					Next
					; Monsters
					For k=0 To NofMonsters-1
						If (MonsterX(k)=i Or MonsterX2(k)=i) And (MonsterY(k)=j Or MonsterY2(k)=j)
							KillMonster(k)
						EndIf
					Next


					
				EndIf
			EndIf
		Next
	Next
				

End Function

; -------------------------------------------
; Buttons
; -------------------------------------------

Function CreateButton(x,y,btype)
	; input tile position x/y
	
	
	
	ButtonEntity(NofButtons)=CreateMesh()
	Surface=CreateSurface(ButtonEntity(NofButtons))
	
	
	AddVertex(Surface,-.5,0,.5, 0+(btype Mod 4)*.25,.25+(btype/4)*.25)
	AddVertex(Surface,.5,0,.5, 0.25+(btype Mod 4)*.25,0.25+(btype/4)*.25)
	AddVertex(Surface,.5,0,-.5, 0.25+(btype Mod 4)*.25,0+(btype/4)*.25)
	AddVertex(Surface,-.5,0,-.5, 0+(btype Mod 4)*.25,0+(btype/4)*.25)
	AddTriangle(Surface,0,1,2)
	AddTriangle(Surface,0,2,3)
	UpdateNormals(ButtonEntity(NofButtons))


	EntityTexture ButtonEntity(NofButtons),ButtonOnTexture
	
;	If btype/3=0 
;		EntityTexture ButtonEntity(NofButtons),ConveyorTexture1
;	Else
;		If btype/3=1
;			EntityColor ButtonEntity(NofButtons),255,0,255
;		Else If btype/3=2
;			EntityColor ButtonEntity(NofButtons),255,255,0
;		Else If btype/3=3
;			EntityColor ButtonEntity(NofButtons),0,255,0
;		Else If btype/3=4
;			EntityColor ButtonEntity(NofButtons),0,255,255
;		EndIf
;
;		;EntityTexture ButtonEntity(NofButtons),LaserGateTexture(btype/3)
;	EndIf

	
		
	
	ScaleEntity ButtonEntity(NofButtons),0.8,1,0.8
		
	ButtonType(NofButtons)=btype
	
	PositionEntity ButtonEntity(NofButtons),x+0.5,0.01,-y-0.5	
	
	ButtonX(NofButtons)=x
	ButtonY(NofButtons)=y
	
	ButtonTimer(NofButtons)=0
	ButtonActive(NofButtons)=False	
	
	NofButtons=NofButtons+1
	
	
	
	
	
End Function

Function StepOnButton(x,y)

	; first figure out which button was pushed
	For i=0 To NofButtons-1
		If x=ButtonX(i) And y=ButtonY(i)
			; got it!
			
			
			; Activate if necessary
			If ButtonActive(i)=False 
				ActivateButton(i,0)
				PlayMySound(SFXButton,0)
			EndIf
			
						
			If ButtonType(i) Mod 3 = 2 		; timer
				ButtonTimer(i)=ButtonBaseTimer
				PlayMySound(SFXTimerButton,0)

			EndIf					
					
		EndIf
	Next
	
		
End Function

Function StepOffButton(x,y)

	; first figure out which button was stepped off
	For i=0 To NofButtons-1
		If x=ButtonX(i) And y=ButtonY(i)
			; got it!
			
			; Act/DeActivate if flip switch
			If ButtonType(i) Mod 3 =1 
				If ButtonActive(i)=True And CheckButtonType(ButtonType(i))=False
					DeActivateButton(i,0)
				EndIf
				
			EndIf
			
				
					
		EndIf
	Next
	
		
End Function

Function CheckButtons()
	; checks all buttons if something is on it and activates (done at start of level)
	For i=0 To NofButtons-1
		; take each button and check if something is on it
		If CheckButton(i)=True Then StepOnButton(ButtonX(i),ButtonY(i))
	Next
End Function
Function CheckButtonType(btype)
	; checks a particular type of button (used for stepping off round buttons)
	For i=0 To NofButtons-1
		If ButtonType(i)=btype
			If CheckButton(i)=True
				Return True
			EndIf
		EndIf
	Next
	Return False
End Function
Function CheckButton(i)
	; check a particular button (used by the above functions)
	
	; stinky/loof
	If (StinkyX=ButtonX(i) And StinkyY=ButtonY(i) And StinkyDead=False)
		Return True
	EndIf
	If  (LoofX=ButtonX(i) And LoofY=ButtonY(i) And LoofExists=True And LoofDead=False)
		Return True
	EndIf
	; boxes
	For j=0 To NofBoxes-1
		If ButtonX(i)=BoxX(j) And ButtonY(i)=BoxY(j) And BoxDead(j)=False
			Return True
		EndIf
	Next
	; monsters
	For j=0 To Nofmonsters-1
		If ButtonX(i)=monsterX(j) And ButtonY(i)=monsterY(j) And monsterdying(j)=False
			Return True
		EndIf
	Next

	Return False
End Function
Function ActivateButton(i,iteration)

	ButtonActive(i)=True

	;TranslateEntity ButtonEntity(i),0,-0.019,0
	EntityTexture ButtonEntity(i),ButtonOffTexture
	
		
	; what kind (i.e color/transporter) of button
	If ButtonType(i)/3 = 0 ; transporter
		If ConveyorActive=True
			ConveyorActive=False
		Else 
			ConveyorActive=True
		EndIf
	Else ; lasergate
		ToggleLaserGate(ButtonType(i)/3,True)
	EndIf
	
	If iteration=0
		; and activate all other buttons of the same type
		For j=0 To NofButtons-1
			If ButtonType(j)=ButtonType(i)
				;ActivateButton(j,1)
				ButtonActive(j)=True
				EntityTexture ButtonEntity(j),ButtonOffTexture

			EndIf
		Next
	EndIf


End Function

Function DeActivateButton(i,iteration)

	ButtonActive(i)=False

;	If ButtonType(i)/3=0 
		EntityTexture ButtonEntity(i),ButtonOnTexture
;	Else
;		If ButtonType(i)/3=1
;			EntityColor ButtonEntity(i),255,0,255
;		Else If ButtonType(i)/3=2
;			EntityColor ButtonEntity(i),255,255,0
;		Else If ButtonType(i)/3=3
;			EntityColor ButtonEntity(i),0,255,0
;		Else If ButtonType(i)/3=4
;			EntityColor ButtonEntity(i),0,255,255
;		EndIf

		;EntityTexture ButtonEntity(i),LaserGateTexture(ButtonType(i)/3)
;	EndIf
	
	; what kind (i.e color/transporter) of button
	If ButtonType(i)/3 = 0 ; transporter
		If ConveyorActive=True
			ConveyorActive=False
		Else 
			ConveyorActive=True
		EndIf
	Else  ; lasergate
	
		
		ToggleLaserGate(ButtonType(i)/3,False)
				
		
	EndIf
	
	
	If iteration=0
		; and activate all other buttons of the same type
		For j=0 To NofButtons-1
			If ButtonType(j)=ButtonType(i)
				;DeActivateButton(j,1)
				ButtonActive(j)=False
				EntityTexture ButtonEntity(j),ButtonOnTexture

			EndIf
		Next
	EndIf


End Function

Function ControlTimerButtons()
	
	For i=0 To NofButtons-1
		If ButtonType(i) Mod 3 = 2 And ButtonActive(i)=True
			; ticking down
			ButtonTimer(i)=ButtonTimer(i)-1
			RotateEntity ButtonEntity(i),0,(ButtonTimer(i)*90)/ButtonBaseTimer,0
			If ButtonTimer(i)<=0
				RotateEntity ButtonEntity(i),0,0,0
				ButtonActive(i)=False
				DeActivateButton(i,0)
			EndIf
		EndIf
	Next
End Function

; -------------------------------------------
; Teleporters
; -------------------------------------------

Function StinkyStepOnTeleporter(x,y,dx,dy)
	
	; returns true if movement was possible
	
	ttype=MapTile(x,y)-1200
	x2=-1
	; first find out where the other teleporter is located
	For i=0 To MapWidth-1
		For j=0 To MapHeight-1
			If MapTile(i,j)-1200=ttype And (x<>i Or y<>j)
				; found it
				x2=i
				y2=j
			EndIf
		Next
	Next
	If x2=-1
		; found nothing - shouldn't happen though
		Return False
	EndIf
	
	; i/j is now the new teleporter position
	; move Stinky to new position and check four directions of movement
	StinkyX=x2
	StinkyY=y2
	PositionEntity StinkyEntity,x2+0.5,0,-(y2+0.5)
	PositionEntity StinkyShadowEntity,x2+0.2,0.01,-(y2+0.2)

	
	If CanStinkyMove(dx,dy)=True
		MoveStinky(dx,dy)
	Else If CanStinkyMove(-dx,-dy)=True
		MoveStinky(-dx,-dy)
	Else If CanStinkyMove(dy,dx)=True
		MoveStinky(dy,dx)
	Else If CanStinkyMove(-dy,-dx)=True
		MoveStinky(-dy,-dx)
	Else 
		; couldn't move anywhere - move back
		StinkyX=x
		StinkyY=y
	
		PositionEntity StinkyEntity,x+0.5,0,-(y+0.5)
		PositionEntity StinkyShadowEntity,x+0.2,0.01,-(y+0.2)	
		
		PlayMySound(SFXHmmm,0)
		Return False

	EndIf

	PlayMySound(SFXTeleport,0)
	If FocusOnStinky=True	
		CameraDX=EntityX(StinkyEntity)-EntityX(Camera)
		CameraDZ=(EntityZ(StinkyEntity)-9)-EntityZ(Camera)
	EndIf
	
	Return True
	

End Function
Function LoofStepOnTeleporter(x,y,dx,dy)
	
	; returns true if movement was possible
	
	ttype=MapTile(x,y)-1200
	x2=-1
	; first find out where the other teleporter is located
	For i=0 To MapWidth-1
		For j=0 To MapHeight-1
			If MapTile(i,j)-1200=ttype And (x<>i Or y<>j)
				; found it
				x2=i
				y2=j
			EndIf
		Next
	Next
	If x2=-1
		; found nothing - shouldn't happen though
		Return False
	EndIf
	
	; i/j is now the new teleporter position
	; move Loof to new position and check four directions of movement
	LoofX=x2
	LoofY=y2
	PositionEntity LoofEntity,x2+0.5,0,-(y2+0.5)
	PositionEntity LoofShadowEntity,x2+0.25,0.01,-(y2+0.25)	
	
	If CanLoofMove(dx,dy)=True
		MoveLoof(dx,dy)
	Else If CanLoofMove(-dx,-dy)=True
		MoveLoof(-dx,-dy)
	Else If CanLoofMove(dy,dx)=True
		MoveLoof(dy,dx)
	Else If CanLoofMove(-dy,-dx)=True
		MoveLoof(-dy,-dx)
	Else 
		; couldn't move anywhere - move back
		LoofX=x
		LoofY=y
	
		PositionEntity LoofEntity,x+0.5,0,-(y+0.5)	
		PositionEntity LoofShadowEntity,x+0.25,0.01,-(y+0.25)	

		PlayMySound(SFXHmmm,0)

		Return False

	EndIf
	
	PlayMySound(SFXTeleport,0)
	If FocusOnStinky=False
		CameraDX=EntityX(LoofEntity)-EntityX(Camera)
		CameraDZ=(EntityZ(LoofEntity)-9)-EntityZ(Camera)
	EndIf

	
	Return True
	

End Function



Function BoxStepOnTeleporter(i,x,y,dx,dy)
	
	; returns true if movement was possible
	
	ttype=MapTile(x,y)-1200
	x2=-1
	; first find out where the other teleporter is located
	For i2=0 To MapWidth-1
		For j=0 To MapHeight-1
			If MapTile(i2,j)-1200=ttype And (x<>i2 Or y<>j)
				; found it
				x2=i2
				y2=j
			EndIf
		Next
	Next
	If x2=-1
		; found nothing - shouldn't happen though
		Return False
	EndIf
	
	; i/j is now the new teleporter position
	; move Box to new position and check four directions of movement
	BoxX(i)=x2
	BoxY(i)=y2
	PositionEntity BoxEntity(i),x2+0.5,0.4,-(y2+0.5)	
	BoxMoving(i)=False
	If CanBoxMove(i,dx,dy)=True
		MoveBox(i,dx,dy,0)
	Else If CanBoxMove(i,-dx,-dy)=True
		MoveBox(i,-dx,-dy,0)
	Else If CanBoxMove(i,dy,dx)=True
		MoveBox(i,dy,dx,0)
	Else If CanBoxMove(i,-dy,-dx)=True
		MoveBox(i,-dy,-dx,0)
	Else 
		; couldn't move anywhere - move back
		BoxX(i)=x
		BoxY(i)=y
	
		PositionEntity BoxEntity(i),x+0.5,0.4,-(y+0.5)	
		
		Return False

	EndIf
	
	; and move the box already a little bit so it is ahead of stinky and
	; 	can be pushed if stinky follows
	TranslateEntity BoxEntity(i),BoxMovingX(i)/Float(BoxSpeed(i)),0,-BoxMovingY(i)/Float(BoxSpeed(i))
	BoxMovingTimer(i)=BoxMovingTimer(i)-1
	
	Return True
	

End Function

; -------------------------------------------
; Monsters
; -------------------------------------------

Function CreateMonster(x,y,btype,dir)
	
	; btype	0 - coily
	;		1,2 - z-bot
	;		3,4 - bomber
	
	; First the Mesh
	If btype=2 Or btype=4	; left/right variations: same mesh
		MonsterEntity(NofMonsters)=CopyEntity(MonsterMesh(btype-1)) 
	Else
		MonsterEntity(NofMonsters)=CopyEntity(MonsterMesh(btype)) 
	EndIf
	MonsterShadowEntity(NofMonsters)=CreateFlatSquare(RoundShadowTexture)
	PositionMesh MonsterShadowEntity(NofMonsters),-.5,0,-.5
	
	; Position of Mesh/Shadow	
	If btype=0
		PositionEntity MonsterEntity(NofMonsters),x+0.5,0.4,-(y+0.5)
		TurnEntity MonsterEntity(NofMonsters),0,Rand(1,300),0
		ScaleMesh MonsterShadowEntity(NofMonsters),.6,1,.6
	Else If btype=1 Or btype=2
		PositionEntity MonsterEntity(NofMonsters),x+.5,.3,-(y+.5)
		RotateEntity MonsterEntity(NofMonsters),-90,0,0
		ScaleMesh MonsterShadowEntity(NofMonsters),.4,1,.4
	Else If btype=3 Or btype=4
		PositionEntity MonsterEntity(NofMonsters),x+0.5,0,-(y+0.5)
		ScaleMesh MonsterShadowEntity(NofMonsters),.7,1,.7
	EndIf
	PositionEntity MonsterShadowEntity(NofMonsters),x+.5,0.01,-y-.0
	If btype=1 Or btype=2
		PositionEntity MonsterShadowEntity(NofMonsters),x+.5,0.01,-y-.3
	EndIf
	
	MonsterX(NofMonsters)=x
	MonsterY(NofMonsters)=y
	MonsterX2(NofMonsters)=x
	MonsterY2(NofMonsters)=y
	MonsterMoving(NofMonsters)=False
	MonsterType(NofMonsters)=btype
	
	; direction
	If dir=0
		MonsterMovingX(NofMonsters)=0
		MonsterMovingY(NofMonsters)=-1
		If btype>2 
			TurnEntity MonsterEntity(NofMonsters),0,180,0
		Else If btype>0
			TurnEntity MonsterEntity(NofMonsters),0,0,-90
		EndIf
	Else If dir=1
		MonsterMovingX(NofMonsters)=1
		MonsterMovingY(NofMonsters)=0
		If btype>2 
			TurnEntity MonsterEntity(NofMonsters),0,90,0
		EndIf
	Else If dir=2
		MonsterMovingX(NofMonsters)=0
		MonsterMovingY(NofMonsters)=1
		If btype>0
			TurnEntity MonsterEntity(NofMonsters),0,0,90
		EndIf
	Else 
		MonsterMovingX(NofMonsters)=-1
		MonsterMovingY(NofMonsters)=0
		If btype>2 
			TurnEntity MonsterEntity(NofMonsters),0,-90,0
		Else If btype>0
			TurnEntity MonsterEntity(NofMonsters),0,0,180
		EndIf
	EndIf
	If btype=0 ; beachball - no dir
		MonsterMovingX(NofMonsters)=0
		MonsterMovingY(NofMonsters)=0
	EndIf
	
	MonsterDead(NofMonsters)=False
	MonsterDying(NofMonsters)=False
	MonsterDyingTimer(NofMonsters)=0
	
	If btype=0	
		MonsterSpeed(NofMonsters)=BaseSpeed*2
	Else If btype=1 Or btype=2
		MonsterSpeed(NofMonsters)=BaseSpeed*2/3
	Else
		MonsterSpeed(NofMonsters)=BaseSpeed*3/2
	EndIf
	
	; Special Data
	For i=0 To 9
		MonsterData(NofMonsters,i)=0
	Next
	
	If btype=0
		MonsterData(NofMonsters,0)=-MonsterSpeed(NofMonsters)/2	; Bouncing Timer
		MonsterData(NofMonsters,1)=Rand(-16,16)	; turning speed (*8)
	Else If btype=1 Or btype=2
		;MonsterData(NofMonsters,1)  RobotCurrentRotation (for turning while walking)
		If MonsterMovingX(NofMonsters)=0
			If MonsterMovingY(NofMonsters)=1	
				MonsterData(NofMonsters,1)=180
			Else 
				MonsterData(NofMonsters,1)=0
			EndIf
		Else
			If MonsterMovingX(NofMonsters)=1
				MonsterData(NofMonsters,1)=270
			Else 
				MonsterData(NofMonsters,1)=90
			EndIf
		EndIf

	
	Else If btype=3 Or btype=4
		MonsterData(NofMonsters,0)=0 ; logic: turning timer to switch to straight
		
		;MonsterData(NofMonsters,1)  RobotCurrentRotation (for turning while walking)
		If MonsterMovingX(NofMonsters)=0
			If MonsterMovingY(NofMonsters)=1	
				MonsterData(NofMonsters,1)=180
			Else 
				MonsterData(NofMonsters,1)=0
			EndIf
		Else
			If MonsterMovingX(NofMonsters)=1
				MonsterData(NofMonsters,1)=270
			Else 
				MonsterData(NofMonsters,1)=90
			EndIf
		EndIf
		
		AnimateMD2 MonsterEntity(NofMonsters),1,.4,1,24
	
	
	EndIf
	NofMonsters=NofMonsters+1
	
	

End Function

Function ControlMonsters()

	For i=0 To NofMonsters-1
		
		If MonsterType(i)=0
			ControlBeachBall(i)
		Else If MonsterType(i)=1 Or MonsterType(i)=2
			ControlRobot(i)
		Else
			ControlBomber(i)
		EndIf
	Next 

End Function

Function ControlBeachBall(i)


	; first check if dying first
	If MonsterDying(i)=True
		; yep, do a deflation animation
		
		; shrink it
		ScaleEntity MonsterEntity(i),.4*Float(MonsterDyingTimer(i))/Float(BaseSpeed*4),.38,.4*Float(MonsterDyingTimer(i))/Float(BaseSpeed*4)
		
		; move it
		MoveEntity MonsterEntity(i),0,.1,0
		If MonsterDyingTimer(i)>BaseSpeed*3
			TurnEntity MonsterEntity(i),0,0,4		
		Else
			TurnEntity MonsterEntity(i),0,0,-4
		EndIf
		
		; finally, remove entity if all gone
		MonsterDyingTimer(i)=MonsterDyingTimer(i)-1
		If MonsterDyingTimer(i)=0
			MonsterDead(i)=True
		EndIf
	
		Return ; don't do anything else
	EndIf

	; animation
	; Turn
	TurnEntity MonsterEntity(i),0,Float(MonsterData(i,1))/8.0,0
	
	; Bounce
	If MonsterData(i,0)>-MonsterSpeed(i)/2
		TranslateEntity MonsterEntity(i),0,Float(MonsterData(i,0))/500.0,0
		TranslateEntity MonsterShadowEntity(i),Float(MonsterData(i,0))/1000.0,0,-Float(MonsterData(i,0))/1000.0
		
		ScaleEntity MonsterEntity(i),.5,.5-.4*Float(Abs(MonsterData(i,0)))/Float(MonsterSpeed(i)/2),.5

		MonsterData(i,0)=MonsterData(i,0)-1
	EndIf
	
	

	; check collision with Stinky
	If StinkyDying=False And StinkyDead=False
		If MonsterMoving(i)=False Or (MonsterMoving(i)=True And MonsterMovingTimer(i)>=MonsterSpeed(i)/2)
			x=MonsterX(i)
			y=MonsterY(i)
		Else
			x=MonsterX2(i)
			y=MonsterY2(i)
		EndIf
	 	If StinkyMoving=False Or (StinkyMoving=True And StinkyMovingTimer>=StinkySpeed/2)
			x2=StinkyX
			y2=StinkyY
		Else
			x2=StinkyX2
			y2=StinkyY2
		EndIf
		If x=x2 And y=y2
			KillStinky()
		EndIf
	EndIf
	; check collision with Loof
	If (LoofExists=True And LoofDying=False And LoofDead=False)
		If LoofMoving=False Or (LoofMoving=True And LoofMovingTimer>=StinkySpeed/2)
			x2=LoofX
			y2=LoofY
		Else
			x2=LoofX2
			y2=LoofY2
		EndIf
		If x=x2 And y=y2
			KillLoof()
		EndIf
	EndIf




	; now check if movement pattern is to be changed
	If MonsterMoving(i)=True Return

	dir=Rand(-1,1)
	If dir=0
		If Rand(100)<50
			dir=-1
		Else
			dir=1
		EndIf
		If CanMonsterMove(i,0,dir)=True 
			If DoesSoundExist= True Then SoundPitch SFXBounce,20000+Rand(0,4000)
			PlayMySound(SFXBounce,MonsterEntity(i))
			MoveMonster(i,0,dir)
			MonsterData(i,0)=MonsterSpeed(i)/2-1
		EndIf
	Else
		If CanMonsterMove(i,dir,0)=True 
			If DoesSoundExist= True Then SoundPitch SFXBounce,20000+Rand(0,4000)
			PlayMySound(SFXBounce,MonsterEntity(i))
			MoveMonster(i,dir,0)
			MonsterData(i,0)=MonsterSpeed(i)/2-1
		EndIf
	EndIf

End Function

Function ControlRobot(i)
	; first check if dying first
	If MonsterDying(i)=True
		MonsterDead(i)=True
	
		Return ; don't do anything else
	EndIf

	; check collision with Stinky
	If StinkyDying=False And StinkyDead=False
		If MonsterMoving(i)=False Or (MonsterMoving(i)=True And MonsterMovingTimer(i)>=MonsterSpeed(i)/2)
			x=MonsterX(i)
			y=MonsterY(i)
		Else
			x=MonsterX2(i)
			y=MonsterY2(i)
		EndIf
	 	If StinkyMoving=False Or (StinkyMoving=True And StinkyMovingTimer>=StinkySpeed/2)
			x2=StinkyX
			y2=StinkyY
		Else
			x2=StinkyX2
			y2=StinkyY2
		EndIf
		If x=x2 And y=y2
			KillStinky()
		EndIf
	EndIf
	; check collision with Loof
	If (LoofExists=True And LoofDying=False And LoofDead=False)
		If LoofMoving=False Or (LoofMoving=True And LoofMovingTimer>=StinkySpeed/2)
			x2=LoofX
			y2=LoofY
		Else
			x2=LoofX2
			y2=LoofY2
		EndIf
		If x=x2 And y=y2
			KillLoof()
		EndIf
	EndIf


	; turning find goal
	If MonsterMovingX(i)=0
		If MonsterMovingY(i)=1	
			RobotRotationGoal=90
		Else 
			RobotRotationGoal=270
		EndIf
	Else
		If MonsterMovingX(i)=1
			RobotRotationGoal=0
		Else 
			RobotRotationGoal=180
		EndIf
	EndIf
	; turning into walk-direction
	If MonsterData(i,1)<>RobotRotationGoal
		If MonsterData(i,1)<RobotRotationGoal And RobotRotationGoal-MonsterData(i,1)<180
			MonsterData(i,1)=MonsterData(i,1)+6
		Else If MonsterData(i,1)>RobotRotationGoal And MonsterData(i,1)-RobotRotationGoal>180
			MonsterData(i,1)=MonsterData(i,1)+6
	  	Else
			MonsterData(i,1)=MonsterData(i,1)-6
		EndIf
		If MonsterData(i,1)=360 Then MonsterData(i,1)=0
		If MonsterData(i,1)=-6 Then MonsterData(i,1)=354
	EndIf

	RotateEntity MonsterEntity(i),-90,0,MonsterData(i,1)



	; now check if movement pattern is to be changed
	If MonsterMoving(i)=True Return

	; default - move to left/right
	If MonsterMovingX(i)=0 And MonsterMovingY(i)=0
		If MonsterType(i)=1
			MonsterMovingX(i)=-1
		Else 
			MonsterMovingX(i)=1
		EndIf
	EndIf
	
	; can it continue in current direction?
	If CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
		MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
	Else
		;no, need to change direction
		If MonsterType(i)=1 ; left-handed movement
			; Try a turn to the left first
			k=MonsterMovingX(i)
			MonsterMovingX(i)=MonsterMovingY(i)
			MonsterMovingY(i)=-k
			If CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
				MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
			Else
				; try a turn to the right next
				MonsterMovingX(i)=-MonsterMovingX(i)
				MonsterMovingY(i)=-MonsterMovingY(i)
				If CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
					MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
				Else
					; try backing up last
					k=MonsterMovingX(i)
					MonsterMovingX(i)=-MonsterMovingY(i)
					MonsterMovingY(i)=k
					If CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
						MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
					EndIf
				EndIf
			EndIf
		Else ; right-handed movement
			; Try a turn to the right first
			k=MonsterMovingX(i)
			MonsterMovingX(i)=-MonsterMovingY(i)
			MonsterMovingY(i)=k
			If CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
				MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
			Else
				; try a turn to the right next
				MonsterMovingX(i)=-MonsterMovingX(i)
				MonsterMovingY(i)=-MonsterMovingY(i)
				If CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
					MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
				Else
					; try backing up last
					k=MonsterMovingX(i)
					MonsterMovingX(i)=MonsterMovingY(i)
					MonsterMovingY(i)=-k
					If CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
						MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
					EndIf
				EndIf
			EndIf
		EndIf

	EndIf
	
	



								
	


End Function
Function ControlBomber(i)

	If MonsterDead(i)=True Return
	
	;TurnEntity MonsterEntity(i),0,1,0
	
	Flag=False
	
	; check for stinky/loof around it
	If MonsterMoving(i)=False Or (MonsterMoving(i)=True And MonsterMovingTimer(i)>=MonsterSpeed(i)/2)
		x2=MonsterX(i)
		y2=MonsterY(i)
	Else
		x2=MonsterX2(i)
		y2=MonsterY2(i)
	EndIf
	
	If StinkyMoving=False Or (StinkyMoving=True And StinkyMovingTimer>=StinkySpeed/2)
		xs=StinkyX
		ys=StinkyY
	Else
		xs=StinkyX2
		ys=StinkyY2
	EndIf

	
	If LoofMoving=False Or (LoofMoving=True And LoofMovingTimer>=LoofSpeed/2)
		xl=LoofX
		yl=LoofY
	Else
		xl=LoofX2
		yl=LoofY2
	EndIf


	For x3=-1 To 1 Step 2
		
		x=x2+x3
		y=y2

		If StinkyDying=False And StinkyDead=False
			If x=xs And y=ys
				KillStinky()
				Flag=True				
			EndIf
		EndIf
		If LoofExists=True And LoofDying=False And LoofDead=False
			If x=xl And y=yl
				KillLoof()
				Flag=True
			EndIf
		EndIf
		
		x=x2
		y=y2+x3

		If StinkyDying=False And StinkyDead=False
			If x=xs And y=ys
				KillStinky()
				Flag=True				
			EndIf
		EndIf
		If LoofExists=True And LoofDying=False And LoofDead=False
			If x=xl And y=yl
				KillLoof()
				Flag=True
			EndIf
		EndIf

		
	Next
	If Flag=True 
		KillMonster(i)
		Return
	EndIf



	; turning find goal
	If MonsterMovingX(i)=0
		If MonsterMovingY(i)=1	
			RobotRotationGoal=180
		Else 
			RobotRotationGoal=0
		EndIf
	Else
		If MonsterMovingX(i)=1
			RobotRotationGoal=270
		Else 
			RobotRotationGoal=90
		EndIf
	EndIf
	; turning into walk-direction
	If MonsterData(i,1)<>RobotRotationGoal
		If MonsterData(i,1)<RobotRotationGoal And RobotRotationGoal-MonsterData(i,1)<180
			MonsterData(i,1)=MonsterData(i,1)+3
		Else If MonsterData(i,1)>RobotRotationGoal And MonsterData(i,1)-RobotRotationGoal>180
			MonsterData(i,1)=MonsterData(i,1)+3
	  	Else
			MonsterData(i,1)=MonsterData(i,1)-3
		EndIf
		If MonsterData(i,1)=360 Then MonsterData(i,1)=0
		If MonsterData(i,1)=-3 Then MonsterData(i,1)=357
	EndIf

	RotateEntity MonsterEntity(i),0,MonsterData(i,1),0






	; now check if movement pattern is to be changed
	If MonsterMoving(i)=True Return

	; default - move to right
	If MonsterMovingX(i)=0 And MonsterMovingY(i)=0
		MonsterMovingX(i)=1
	EndIf
	
	OldX=MonsterMovingX(i)
	OldY=MonsterMovingY(i)
	
	If MonsterType(i)=3	;left-handed movement
		
		; turn right
		k=MonsterMovingX(i)
		MonsterMovingX(i)=-MonsterMovingY(i)
		MonsterMovingY(i)=k
		If MonsterData(i,0)<4 And CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
			MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
			MonsterData(i,0)=MonsterData(i,0)+1
		Else
			MonsterData(i,0)=0
			; can't turn left, try continuing
			k=MonsterMovingX(i)
			MonsterMovingX(i)=MonsterMovingY(i)
			MonsterMovingY(i)=-k
			If CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
				MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
			Else
				; can't, try turning left
				k=MonsterMovingX(i)
				MonsterMovingX(i)=MonsterMovingY(i)
				MonsterMovingY(i)=-k
				If CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
					MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
				Else
					; can't, try backing up
					k=MonsterMovingX(i)
					MonsterMovingX(i)=MonsterMovingY(i)
					MonsterMovingY(i)=-k
					If CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
						MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
					Else
						; can't go anywhere - boom!
						KillMonster(i)
						Return
						
					EndIf
				EndIf
			EndIf
		EndIf
	EndIf

	
	If MonsterType(i)=4 ; right-handed movement
		
		; turn left
		k=MonsterMovingX(i)
		MonsterMovingX(i)=MonsterMovingY(i)
		MonsterMovingY(i)=-k
		If MonsterData(i,0)<4 And CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
			MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
			MonsterData(i,0)=MonsterData(i,0)+1
		Else
			MonsterData(i,0)=0
			; can't turn left, try continuing
			k=MonsterMovingX(i)
			MonsterMovingX(i)=-MonsterMovingY(i)
			MonsterMovingY(i)=k
			If CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
				MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
			Else
				; can't, try turning right
				k=MonsterMovingX(i)
				MonsterMovingX(i)=-MonsterMovingY(i)
				MonsterMovingY(i)=k
				If CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
					MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
				Else
					; can't, try backing up
					k=MonsterMovingX(i)
					MonsterMovingX(i)=-MonsterMovingY(i)
					MonsterMovingY(i)=k
					If CanMonsterMove(i,MonsterMovingX(i),MonsterMovingY(i))=True
						MoveMonster(i,MonsterMovingX(i),MonsterMovingY(i))
					Else
						; can't go anywhere - boom!
						KillMonster(i)
						Return
						
					EndIf
				EndIf
			EndIf
		EndIf
	EndIf
	
	
	

		


End Function

Function CanMonsterMove(i,dx,dy)
	; defaults to true, but checks if anything is in the way
	
	x=MonsterX(i)+dx
	y=MonsterY(i)+dy
	
	Tile=MapTile(x,y)
	; can only walk on floor, electrotiles, open gates, buttons
	If Tile/100 = 1 Or Tile/100=7 Or (Tile/100=10 And Tile Mod 100>=50) Or Tile/100=11 Or Tile/100=13
		;ok
	Else If (tile/100=3 Or tile/100=9) And (MonsterType(i)=1 Or MonsterType(i)=2)
		;also ok - robot can walk into water (but will then explode)
	Else
		Return False
	EndIf
	
	
	
	; now check boxes - can't move into any
	For j=0 To NofBoxes-1
		If (x=BoxX(j) Or x=BoxX2(j)) And (y=BoxY(j) Or y=BoxY2(j))
			Return False
		EndIf
	Next
	
	; now check other monsters - can't move into any
	For j=0 To NofMonsters-1
		If (x=MonsterX(j) Or x=MonsterX2(j)) And (y=MonsterY(j) Or y=MonsterY2(j)) And i<>j
			Return False
		EndIf
	Next
	
	; now check items - can't move into any
	For j=0 To NofItems-1
		If x=ItemX(j) And y=ItemY(j)
			Return False
		EndIf
	Next


	
	
	
	Return True
	
End Function

Function MoveMonster(i,dx,dy)

	

	MonsterMovingTimer(i)=MonsterSpeed(i)
	MonsterX2(i)=MonsterX(i)+dx
	MonsterY2(i)=MonsterY(i)+dy
	MonsterMovingX(i)=dx
	MonsterMovingY(i)=dy

	
	; Check if Stinky is in the way, if so: boom!
	;If (BoxX2(i)=StinkyX And BoxY2(i)=StinkyY) Or (BoxX2(i)=StinkyX2 And BoxY2(i)=StinkyY2)
		;unless stinky is moving away, [LATER: or sideways at end of move]
		;(also discount conveyors, since stinky could be 'walking' against it,
		; perhaps later look at this more precisely, for now it should do)
	;	If StinkyMoving=True And StinkyMovingX=dx And StinkyMovingY=dy And MapTile(StinkyX,StinkyY)/100<>5
			; no interaction
	;	Else
			; boom
	;		KillStinky()
	;	EndIf
	;EndIf


	MonsterMoving(i)=True

	

End Function

Function EndMoveMonster(i)

	Oldx=MonsterX(i)
	OldY=MonsterY(i)

	
	
	MonsterX(i)=MonsterX2(i)
	MonsterY(i)=MonsterY2(i)
	
	; check old tile
	If MapTile(Oldx,Oldy)/100=11 ;buttons
		StepOffButton(Oldx,Oldy)
	EndIf

	MonsterMoving(i)=False
	
	; check new tile
	If MapTile(MonsterX(i),MonsterY(i))/100=11 ;buttons
		StepOnButton(MonsterX(i),MonsterY(i))
	EndIf
	If MapTile(MonsterX(i),MonsterY(i))/100=3 Or MapTile(MonsterX(i),MonsterY(i))/100=9; water
		KillMonster(i)
	EndIf

	
End Function

Function KillMonster(i)

	MonsterDying(i)=True

	; stepped off button
	If MapTile(MonsterX(i),MonsterY(i))/100=11
		StepOffButton(MonsterX(i),MonsterY(i))
	EndIf

	; sound + bonus?
	If MonsterType(i)=0
		PlayMySound(SFXCoilyDeath,MonsterEntity(i))
		CurrentLevelScore=CurrentLevelScore+250
		CreateParticle(0,9,0.5,MonsterX(i)+.5,1,-MonsterY(i)-.2,0,.01,0,0,0,120)
	EndIf
	If MonsterType(i)>0 ;=3 Or MonsterType(i)=4
		PlayMySound(SFXBombExplo,MonsterEntity(i))
		CurrentLevelScore=CurrentLevelScore+250
		CreateParticle(0,9,0.5,MonsterX(i)+.5,1,-MonsterY(i)-.2,0,.01,0,0,0,120)
	EndIf


	; Remove the entity logically first, actual death is taken care of later
	
	FreeEntity MonsterShadowEntity(i)
	;TurnEntity MonsterEntity(i),0,0,Rand(-30,30) 
	;If MonsterType(i)=0
		MonsterDyingTimer(i)=BaseSpeed*4
	;Else
	;	MonsterDyingTimer(i)=BaseSpeed*3
	;EndIf
	
	If MonsterType(i)>2
		; blow up immediately
		MonsterDead(i)=True
		; create explosion of particles
		For j=1 To 7*DetailLevel
			CreateParticle(0,0,0.2,EntityX(MonsterEntity(i)),EntityY(MonsterEntity(i)),EntityZ(MonsterEntity(i)),Rnd(-.04,.04),Rnd(.01,.06),Rnd(-.04,.04),Rnd(-10,10),.001,60*DetailLevel)
			CreateParticle(0,5,0.2,EntityX(MonsterEntity(i)),EntityY(MonsterEntity(i)),EntityZ(MonsterEntity(i)),Rnd(-.06,.06),Rnd(.03,.09),Rnd(-.06,.06),Rnd(-10,10),.002,60*DetailLevel)

		Next
	
	Else If MonsterType(i)>0
		; blow up immediately
		MonsterDead(i)=True
		; create explosion of particles
		For j=1 To 14*DetailLevel
			;CreateParticle(0,0,0.2,EntityX(MonsterEntity(i)),EntityY(MonsterEntity(i)),EntityZ(MonsterEntity(i)),Rnd(-.04,.04),Rnd(.01,.06),Rnd(-.04,.04),Rnd(-10,10),.001,60*DetailLevel)
			CreateParticle(0,5,0.1,EntityX(MonsterEntity(i)),EntityY(MonsterEntity(i)),EntityZ(MonsterEntity(i)),Rnd(-.16,.16),Rnd(.08,.2),Rnd(-.16,.16),Rnd(-30,30),.004,60*DetailLevel)

		Next
	EndIf

	
	; BOMBER - Kill everything around it
	If MonsterType(i)=3 Or MonsterType(i)=4
		If MonsterMoving(i)=False Or (MonsterMoving(i)=True And MonsterMovingTimer(i)>=MonsterSpeed(i)/2)
			x2=MonsterX(i)
			y2=MonsterY(i)
		Else
			x2=MonsterX2(i)
			y2=MonsterY2(i)
		EndIf

		For x3=-1 To 1 Step 2
			
			x=x2+x3
			y=y2
			For j=0 To NofBoxes-1
				If BoxDead(j)=False 
					If (x=BoxX(j) Or x=BoxX2(j)) And (y=BoxY(j) Or y=BoxY2(j))
						KillBox(j)
					EndIf
				EndIf
			Next
			For j=0 To NofMonsters-1
				If MonsterDead(j)=False And MonsterDying(j)=False
					If (x=MonsterX(j) Or x=MonsterX2(j)) And (y=MonsterY(j) Or y=MonsterY2(j))
						KillMonster(j)
					EndIf
				EndIf
			Next
			If StinkyDying=False And StinkyDead=False
				If (x=StinkyX Or x=StinkyX2) And (y=StinkyY Or y=StinkyY2)
					KillStinky()
				EndIf
			EndIf
			If LoofExists=True And LoofDying=False And LoofDead=False
				If (x=LoofX Or x=LoofX2) And (y=LoofY Or y=LoofY2)
					KillLoof()
				EndIf
			EndIf
			
			x=x2
			y=y2+x3
			For j=0 To NofBoxes-1
				If BoxDead(j)=False 
					If (x=BoxX(j) Or x=BoxX2(j)) And (y=BoxY(j) Or y=BoxY2(j))
						KillBox(j)
					EndIf
				EndIf
			Next
			For j=0 To NofMonsters-1
				If MonsterDead(j)=False And MonsterDying(j)=False
					If (x=MonsterX(j) Or x=MonsterX2(j)) And (y=MonsterY(j) Or y=MonsterY2(j))
						KillMonster(j)
					EndIf
				EndIf
			Next
			If StinkyDying=False And StinkyDead=False
				If (x=StinkyX Or x=StinkyX2) And (y=StinkyY Or y=StinkyY2)
					KillStinky()
				EndIf
			EndIf
			If LoofExists=True And LoofDying=False And LoofDead=False
				If (x=LoofX Or x=LoofX2) And (y=LoofY Or y=LoofY2)
					KillLoof()
				EndIf
			EndIf

				

					
		
		Next
	EndIf

	
	
	MonsterX(i)=0
	MonsterY(i)=0
	MonsterX2(i)=0
	MonsterY2(i)=0
	MonsterMoving(i)=False

	
End Function

Function CleanUpMonsters()
	; removes any dead boxes
	For i=0 To NofMonsters-1
		If MonsterDead(i)=True
			; yep, get rid of it by overwriting it with later boxes
			FreeEntity MonsterEntity(i)
			
			If i<NofMonsters-1 ;unless it is the last one
				For j=i To NofMonsters-1
					MonsterEntity(j)=MonsterEntity(j+1)
					MonsterShadowEntity(j)=MonsterShadowEntity(j+1)
					MonsterType(j)=MonsterType(j+1)
					MonsterX(j)=MonsterX(j+1)
					MonsterY(j)=MonsterY(j+1)
					MonsterX2(j)=MonsterX2(j+1)
					MonsterY2(j)=MonsterY2(j+1)
					MonsterMoving(j)=MonsterMoving(j+1)
					MonsterMovingX(j)=MonsterMovingX(j+1)
					MonsterMovingY(j)=MonsterMovingY(j+1)
					MonsterMovingTimer(j)=MonsterMovingTimer(j+1)
					MonsterSpeed(j)=MonsterSpeed(j+1)
					MonsterDying(j)=MonsterDying(j+1)
					MonsterDyingTimer(j)=MonsterDyingTimer(j+1)
					MonsterDead(j)=MonsterDead(j+1)
					For k=0 To 9
						MonsterData(j,k)=MonsterData(j+1,k)
					Next
				Next
				i=i-1

			EndIf
			NofMonsters=NofMonsters-1
		EndIf
	Next
End Function

; -------------------------------------------
; Items
; -------------------------------------------

Function CreateItem(x,y,btype)

	; input tile position x/y
	
	
	ItemEntity(NofItems)=CopyEntity(ItemMesh(btype)) 
	
	ItemType(NofItems)=btype
	
	PositionEntity ItemEntity(NofItems),x+.5,0.5,-(y+.5)
	
	ItemX(NofItems)=x
	ItemY(NofItems)=y
	ItemDead(NofItems)=False
	
	
	
	
	If btype=0 ; crystal
		;EntityTexture ItemEntity(NofItems),RainbowTexture
		NofCrystals=NofCrystals+1
		ItemShadowEntity(NofItems)=CopyEntity(ItemShadowMesh(btype))
		PositionEntity ItemShadowEntity(NofItems),x+.6,0.01,-(y+.6)
	EndIf	
	If btype=1 ; endgate
		ItemShadowEntity(NofItems)=CreateFlatSquare(RoundShadowTexture)
		PositionEntity ItemShadowEntity(NofItems),x,0.01,-(y)
	EndIf	
	If btype=3 ; bonus
		;EntityTexture ItemEntity(NofItems),RainbowTexture
		ItemShadowEntity(NofItems)=CopyEntity(ItemShadowMesh(0))
		PositionEntity ItemShadowEntity(NofItems),x+.5,0.01,-(y+.55)
	EndIf	


	
	NofItems=NofItems+1

	

End Function

Function ControlItems()

	For i=0 To NofItems-1
	
		; general control
		If ItemType(i)=0	; coin - rotate
			TurnEntity ItemEntity(i),0,-2,0
			TurnEntity ItemShadowEntity(i),0,-2,0
			;size#=.8+.2*Sin(EntityYaw(ItemEntity(i)))
			;ScaleEntity ItemEntity(i),.8*size,.8*size,.5*size

		EndIf
		
		If ItemType(i)=1	; exit - rotate/pulsate
			TurnEntity ItemEntity(i),0,-1,0
			size#=.4+.1*Sin(EntityYaw(ItemEntity(i)))
			ScaleEntity ItemEntity(i),size,size/2,size
			
			ScaleEntity ItemShadowEntity(i),size*2-.2,size,size*2-.2
			PositionEntity ItemShadowEntity(i),ItemX(i)+.6-size,0.01,-ItemY(i)-.6+size
			
			; and create rainbowstar particles
			If Rand(0,100)<5
			
				;px#=ItemX(i)+.5
				;py#=0.5
				;pz#=-(ItemY(i)+.5)
				;pdx#=Rnd(-.008,.008)
				;pdy#=Rnd(.02,.03)
				;pdz#=Rnd(-.008,.008)
				;pddy#=.0005
				;For j=1 To 180
			;		px=px+pdx
			;		py=py+pdy
			;		pz=pz+pdz
			;		pdy=pdy-pddy
			;	Next
			;	CreateParticle(3,px,py,pz,-pdx,Abs(pdy),-pdz,Rnd(-5,5),pddy,180)
				CreateParticle(0,3,0.1,ItemX(i)+.5,0.5,-(ItemY(i)+.5),Rnd(-.01,.01),Rnd(.02,.035),Rnd(-.01,.01),Rnd(-5,5),.0005,180)
			EndIf
			
			
		EndIf
		
		If ItemType(i)=3	; bonus coin - rotate
			TurnEntity ItemEntity(i),0,0,-2
			;TurnEntity ItemShadowEntity(i),0,-2,0
			;size#=.8+.2*Sin(EntityYaw(ItemEntity(i)))
			;ScaleEntity ItemEntity(i),.8*size,.8*size,.5*size

		EndIf

			
		
		; check if they are picked up (if moving stinky is halfway on its way to the tile)
		; do this last since items might be gone after killitem

		If ItemX(i)=StinkyX2 And ItemY(i)=StinkyY2 
			If StinkyMoving=True And StinkyMovingTimer<StinkySpeed/2 And BothCompleted Mod 2 =0
				; yep, pick up
				PickUpItem(i,0)
				Return
			EndIf
		EndIf
		If ItemX(i)=LoofX2 And ItemY(i)=LoofY2 
			If LoofMoving=True And LoofMovingTimer<LoofSpeed/2 And BothCompleted/2 = 0
				; yep, pick up
				PickUpItem(i,1)
				Return
			EndIf
		EndIf
		
	
	Next
End Function

Function PickUpItem(i,who)
	; Who is 0 or 1 (stinky/loof)
	
	If ItemType(i)=0	; crystal
		If who=0
			PlayMySound(SFXStinkyCoin,0)
		Else
			PlayMySound(SFXLoofCoin,0)
		EndIf
		CreateParticle(0,3,0.3,ItemX(i)+.5,1,-ItemY(i)-.2,0,.02,0,Rnd(4,8),0,120)

		NofCrystals=NofCrystals-1
		; and open door if none left
		If NofCrystals=0
			PlayMySound(SFXLastCoin,0)
			ToggleLaserGate(0,True)
		EndIf
	EndIf
	If ItemType(i)=1	; the end
		
		If who=0 
			StinkyShrink=60
			PlayMySound(SFXStinkyEnd,0)
			HideEntity StinkyShadowEntity
			BothCompleted=BothCompleted+1
		Else 
			PlayMySound(SFXLoofEnd,0)
			LoofShrink=60
			HideEntity LoofShadowEntity
			BothCompleted=BothCompleted+2
		EndIf
		; is the other still around?
		If LoofExists=False Or BothCompleted=3
			LevelCompleted=1
			LevelCompletedTimer=0
			If Language=0
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
				If MapTime<1 Then MapTime=1
				If NoTimeLimit=1 Then MapTime=1
				ex$=ex$+"# #BONUS SCORE: "+Str$(CurrentLevelScore)+"    TIME BONUS: "+Str$(MapTime*10)
				CurrentLevelScore=CurrentLevelScore+MapTime*10
				ex$=ex$+"#TOTAL SCORE: "+Str$(CurrentLevelScore)
				
				If CurrentPlayerLevelScore(CurrentLevel)>0 
					If CurrentLevelScore>CurrentPlayerLevelScore(CurrentLevel)
						ex$=ex$+"#!!! NEW LEVEL RECORD !!!"
					Else
						ex$=ex$+"#NO NEW RECORD."
					EndIf
				EndIf
			Else If Language=1
				ex$="GUT GEMACHT!!! LEVEL BEENDET!!!"
				If MapTime<1 Then MapTime=1
				If NoTimeLimit=1 Then MapTime=1

				ex$=ex$+"# #BONUS PUNKTE: "+Str$(CurrentLevelScore)+"    ZEIT BONUS: "+Str$(MapTime*10)
				CurrentLevelScore=CurrentLevelScore+MapTime*10
				ex$=ex$+"#PUNKTE INSGESAMMT: "+Str$(CurrentLevelScore)
				
				If CurrentPlayerLevelScore(CurrentLevel)>0 
					If CurrentLevelScore>CurrentPlayerLevelScore(CurrentLevel)
						ex$=ex$+"#!!! NEUER LEVEL REKORD !!!"
					Else
						ex$=ex$+"#LEIDER KEIN NEUER REKORD."
					EndIf
				EndIf
			Else If Language=2
				ex$="BIEN REUSSI!!! NIVEAU COMPLETE!!!"
				If MapTime<1 Then MapTime=1
				If NoTimeLimit=1 Then MapTime=1

				ex$=ex$+"# #SCORE BONUS: "+Str$(CurrentLevelScore)+"    SCORE TEMPS: "+Str$(MapTime*10)
				CurrentLevelScore=CurrentLevelScore+MapTime*10
				ex$=ex$+"#SCORE TOTAL: "+Str$(CurrentLevelScore)
				
				If CurrentPlayerLevelScore(CurrentLevel)>0 
					If CurrentLevelScore>CurrentPlayerLevelScore(CurrentLevel)
						ex$=ex$+"#!!! NOUVEAU RECORD POUR CE NIVEAU !!!"
					Else
						ex$=ex$+"#PAS DE NOUVEAU RECORD."
					EndIf
				EndIf


			
			
			EndIf
			If CurrentLevelScore>CurrentPlayerLevelScore(CurrentLevel) 
				CurrentPlayerLevelScore(CurrentLevel)=CurrentLevelScore
			EndIf
			
			StartSign(ex$,7,2,1)
			
			If CurrentLevel<PlayGroundLevel Then SavePlayer()



		Else 
			SwitchStinkyLoof()
		EndIf
		
	EndIf
	If ItemType(i)=2	; time
		MapTime=MapTime+30
	EndIf
	If ItemType(i)=3	; score
		If who=0
			PlayMySound(SFXStinkyBonus,0)
		Else
			PlayMySound(SFXLoofBonus,0)
		EndIf

		CurrentLevelScore=CurrentLevelScore+250
		CreateParticle(0,9,0.5,ItemX(i)+.5,1,-ItemY(i)-.2,0,.02,0,0,0,120)
	EndIf
		
	
	; and remove
	If ItemType(i)<>1 KillItem(i)			
End Function

Function KillItem(i)

	ItemDead(i)=True
	
End Function

Function CleanUpItems()
	; removes any dead boxes
	For i=0 To NofItems-1
		If ItemDead(i)=True
			; yep, get rid of it by overwriting it with later boxes
			FreeEntity ItemEntity(i)
			If ItemType(i)=0 Or ItemType(i)=1 Or ItemType(i)=3
				FreeEntity ItemShadowEntity(i)
			EndIf
			
			If i<NofItems-1 ;unless it is the last one
				For j=i To NofItems-1
					ItemEntity(j)=ItemEntity(j+1)
					ItemShadowEntity(j)=ItemShadowEntity(j+1)
					ItemType(j)=ItemType(j+1)
					ItemX(j)=ItemX(j+1)
					ItemY(j)=ItemY(j+1)
					ItemDead(j)=ItemDead(j+1)
					
				Next
				i=i-1

			EndIf
			NofItems=NofItems-1
		EndIf
	Next
End Function

Function CreateFish(x,y)

	FishEntity(NofFish)=CopyEntity(FishMesh)
	
	;ScaleEntity FishEntity(NofFish),14,14,14 
		
	PositionEntity FishEntity(NofFish),x+.75,-0.5,-(y+.4)+2
	
	FishTimer(NofFish)=0.0
	FishX(NofFish)=x
	FishY(NofFish)=y
	FishSpeed(NofFish)=Rnd(.5,1.5)
	If Rand(0,100)<50
		; other way
		FishSpeed(NofFish)=-FishSpeed(NofFish)
		TurnEntity FishEntity(NofFish),0,180,0
	EndIf
	
	NofFish=NofFish+1

	

End Function

Function ControlFish()
	For i=0 To NofFish-1
		If FishSpeed(NofFish)>0
			FishTimer(i)=FishTimer(i)+FishSpeed(i)
			If FishTimer(i)>=360 Then FishTimer(i)=FishTimer(i)-360
			PositionEntity FishEntity(i),FishX(i)+.75-1.5*Sin(FishTimer(i)),-0.5,-(FishY(i)+.4)+1.5*Cos(FishTimer(i))
			TurnEntity FishEntity(i),0,FishSpeed(i),0 
		Else
			FishTimer(i)=FishTimer(i)-FishSpeed(i)
			If FishTimer(i)>=360 Then FishTimer(i)=FishTimer(i)-360
			PositionEntity FishEntity(i),FishX(i)+.75+1.5*Sin(FishTimer(i)),-0.5,-(FishY(i)+.4)+1.5*Cos(FishTimer(i))
			TurnEntity FishEntity(i),0,FishSpeed(i),0 
		EndIf
	Next 
		
	
End Function

Function CreatePillar(x,y,btype)

	PillarEntity(NofPillars)=CopyEntity(PillarMesh(btype))
	
	;ScaleEntity FishEntity(NofFish),14,14,14 
		
	PositionEntity PillarEntity(NofPillars),x+.5,0,-(y+.5)

	
	NofPillars=NofPillars+1

	

End Function

	