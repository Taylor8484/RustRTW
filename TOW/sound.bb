Function LoadSounds()

	; Load Sound FX
	;SFXWalk=MyLoadSound("sound\walk.wav",True)
	
		
	SFXLevelLose=MyLoadSound("sound\death2.wav",False)
	;SFXLevelWin=MyLoadSound("sound\win.wav",True)
	SFXBreakAway=MyLoadSound("sound\breakaway.wav",False)
	SFXBoxPush=MyLoadSound("sound\boxpush.wav",False)
	SFXBoxSplosh=MyLoadSound("sound\splosh.wav",False)
	SFXBoxExplo=MyLoadSound("sound\boxexplo.wav",False)
	SFXTeleport=MyLoadSound("sound\teleport.wav",False)
	SFXButton=MyLoadSound("sound\button.wav",False)
	
	SFXBounce=MyLoadSound("sound\bounce.wav",False)
	SFXCoilyDeath=MyLoadSound("sound\coilydeath.wav",False)
	SFXBombExplo=MyLoadSound("sound\bombexplo.wav",False)
	
	SFXSingle(0)=MyLoadSound("sound\electro.wav",False)
	SFXSingle(1)=MyLoadSound("sound\fireshot.wav",False)
	SFXSingle(2)=MyLoadSound("sound\fireexplo.wav",False)
	
	; stinky and loof voices
	SFXStinkycoin=MyLoadSound("sound\stinkycoin.wav",False)
	SFXLoofcoin=MyLoadSound("sound\Loofcoin.wav",False)
	SFXStinkyBonus=MyLoadSound("sound\StinkyBonus.wav",False)
	SFXLoofBonus=MyLoadSound("sound\LoofBonus.wav",False)
	SFXLastCoin=MyLoadSound("sound\lastcoin.wav",False)
	SFXStinkyEnd=MyLoadSound("sound\StinkyEnd.wav",False)
	SFXLoofEnd=MyLoadSound("sound\LoofEnd.wav",False)
;	SFXStinkyIce=MyLoadSound("StinkyIce.wav",False)
;	SFXLoofIce=MyLoadSound("LoofIce.wav",False)
	SFXStinkyConvey=MyLoadSound("sound\StinkyConvey.wav",False)
	SFXLoofConvey=MyLoadSound("sound\LoofConvey.wav",False)
	
	SFXHmmm=MyLoadSound("sound\slhmmm.wav",False)
	SFXTime=MyLoadSound("sound\sltime.wav",False)
	SFXDeath=MyLoadSound("sound\sldeath.wav",False)
	
	SFXTalk(0)=MyLoadSound("sound\getready.wav",False)
	SFXTalk(1)=MyLoadSound("sound\herewego.wav",False)
	SFXTalk(2)=MyLoadSound("sound\cmonletsgo.wav",False)
	SFXTalk(3)=MyLoadSound("sound\letsdoit.wav",False)
	SFXTalk(4)=MyLoadSound("sound\stinkyno.wav",False)
	SFXTalk(5)=MyLoadSOund("sound\introcmon.wav",False)

	SFXMenuSound(0)=MyLoadSound("sound\menumove.wav",False)
	SFXMenuSound(1)=MyLoadSound("sound\type.wav",False)
	For i=0 To 10
		SFXCount(i)=MyLoadSound("sound\sl"+Str$(i)+".wav",False)
	Next
	
	If Version>=10
		SFXTimerButton=MyLoadSound("sound\windup.wav",False)
		SFXIce=MyLoadSound("sound\ice.wav",False)
		For i=0 To 4
			SFXSlIce(i)=MyLoadSound("sound\slice"+Str$(i+1)+".wav",False)
		Next
		SFXLava=MyLoadSound("sound\blub.wav",False)	
	EndIf

	
End Function


Function PlayMySound(bsound,bentity)
	If OptionsSoundLevel=0 Return
	If DoesSoundExist=False Return
	; if no distance specified
	If bentity=0 
		SoundVolume bsound,OptionsSoundLevel
		PlaySound(bsound)
		
		Return
	EndIf

	; calculate distance from sound
	TranslateEntity Camera,0,-10,9
	distance#=EntityDistance (Camera,bentity)
	TranslateEntity Camera,0,10,-9

	a#=3.0 ; minimum radius
	b#=9.0 ; maximum radius
	
	volume#=(b-distance)/(b-a)
	
	If OptionsSoundLevel*volume<=0 Then Return
	
	If volume>1 Then volume=1
	
	
	SoundVolume(bsound,OptionsSoundLevel*volume)
	PlaySound(bsound)	
	


	
End Function

Function PlaySingleSound(x,y,bType)
	If OptionsSoundLevel=0 Return
	If DoesSoundExist=False Then Return

	; just checks volume... at the end plays highest volume sound in UpdateGame

	bentity=CreatePivot()
	TranslateEntity bentity,x+.5,0,-y-0.5
	
	; calculate distance from sound
	TranslateEntity Camera,0,-10,9
	distance#=EntityDistance (Camera,bentity)
	TranslateEntity Camera,0,10,-9

	a#=3.0 ; minimum radius
	b#=9.0 ; maximum radius
	
	volume#=(b-distance)/(b-a)
	
	If volume<=0 Then 
		FreeEntity bentity
		Return
	EndIf
	
	If volume>1 Then volume=1
	
	; now, is it bigger?
	
	If volume>SFXSingleVolume(btype) Then SFXSingleVolume(btype)=volume
	
	FreeEntity bentity
	
End Function