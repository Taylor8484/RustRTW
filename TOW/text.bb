

Function SetupFont()

	Modus=2

	FontTex(0)=MyLoadAnimTexture("res12.dat",2,16,32,0,16,False)
	If FontTex(0)=0 
		Modus=6
		FontTex(0)=MyLoadAnimTexture("res12.dat",6,16,32,0,16,False)

	EndIf
	FontTex(1)=MyLoadAnimTexture("res13.dat",Modus,16,32,0,16,False)
	FontTex(2)=MyLoadAnimTexture("res14.dat",Modus,16,32,0,16,False)
	FontTex(3)=MyLoadAnimTexture("res15.dat",Modus,16,32,0,16,False)
	FontTex(4)=MyLoadAnimTexture("res16.dat",Modus,16,32,0,16,False)
	FontTex(5)=MyLoadAnimTexture("res17.dat",Modus,16,32,0,16,False)

	
	
	
	
End Function

Function AddLetter(Ascii,x#,y#,Alpha#,MyTag)

	l.LetterType = New LetterType
	
	l\Entity = CreateSprite(Camera)
	ScaleSprite l\Entity,.01,.02
	TranslateEntity l\Entity,-.5+x*.02+.01,0.375-y*.04-.02,1
	EntityTexture l\Entity,FontTex((Ascii-32)/16),(Ascii-32) Mod 16
	EntityAlpha l\Entity,Alpha
	
	HideEntity l\Entity
	
	l\Tag=MyTag
	l\Active=False

End Function

Function DisplayText(MyText$,x#,y#,Alpha#,MyTag)
	
	; First go through existing letters and erase all with the same Tag
	For l.LetterType = Each LetterType
		If l\Tag=MyTag
			FreeEntity(l\Entity)
			Delete l
		EndIf
	Next
	
	; Now add the line
	For i=1 To Len(MyText$)
		AddLetter(Asc(Mid$(MyText$,i,1)),x+i-1,y,Alpha#,MyTag)
	Next 

End Function

Function CenterText(MyText$,y#,Alpha#,MyTag)
	
	x#=25.0-Len(MyText$)/2.0
	
	; First go through existing letters and erase all with the same Tag
	For l.LetterType = Each LetterType
		If l\Tag=MyTag
			FreeEntity(l\Entity)
			Delete l
		EndIf
	Next
	
	; Now add the line
	For i=1 To Len(MyText$)
		AddLetter(Asc(Mid$(MyText$,i,1)),x+i-1,y,Alpha#,MyTag)
	Next 

End Function


Function StartSign(MyText$,y#,MyTag,Alpha#)

	; First go through existing letters and erase all with the same Tag
	For l.LetterType = Each LetterType
		If l\Tag=MyTag
			FreeEntity(l\Entity)
			Delete l
		EndIf
	Next
	
	; Now Parse the text and add the lines 
	j=1
	While(j<Len(MyText$))
		If Mid$(MyText$,j,1)="#" Or j=Len(MyText$)
			; display the line
			For i=1 To j-1
				AddLetter(Asc(Mid$(MyText$,i,1)),25.0-Float(j-1)/2.0+i-1,y,Alpha,MyTag)
			Next 
			y=y+1
			; and delete and reset that line
			MyText$=Right$(MyText$,Len(MyText$)-j)
			j=0
		EndIf
		j=j+1
	Wend
	; display final line
	For i=1 To j
		AddLetter(Asc(Mid$(MyText$,i,1)),25.0-Float(j)/2.0+i-1,y,Alpha,MyTag)
	Next 
	
	; light
	If MyTag>=11 And MyTag<=15
		If FocusOnStinky=True
			SignActive=1
		Else
			SignActive=2
		EndIf
		LightColor LevelLight,0,0,0
		AmbientLight 100,100,100
	EndIf



End Function
Function ControlSigns()
	; handles blanding in and out of levelname sign
	
	If LevelTimer>800 Return
	
	
	If LevelTimer<200
		For l.LetterType=Each LetterType
			If l\Tag=10
				EntityAlpha l\Entity,Float(LevelTimer)/200.0
			EndIf
			
		Next
	Else If LevelTimer>600 And LevelTimer<800
		For l.LetterType=Each LetterType
			If l\Tag=10
				EntityAlpha l\Entity,1.0-Float(LevelTimer-600)/200.0
			EndIf
		Next
	Else If LevelTimer=800
		StopSign(10)
	EndIf

	
End Function

Function StopSign(MyTag)
	For l.LetterType=Each LetterType
		If l\Tag=MyTag
			FreeEntity l\Entity
			Delete l
		EndIf
	Next
	
	; light
	If MyTag>=11 And MyTag<=15
		SignActive=0
		LightColor LevelLight,255,255,255
		AmbientLight 255,255,255
	EndIf

End Function


Function CheckSigns()
	If FocusOnStinky=True
		; see if stinky on signs
		For i=0 To 4
			If StinkyX=SignX(i) And StinkyY=SignY(i)
				StartSign(SignText$(i),8,11+i,1)
			EndIf
		Next
	Else
		; see if loof on signs
		For i=0 To 4
			If LoofX=SignX(i) And LoofY=SignY(i)
				StartSign(SignText$(i),8,11+i,1)
			EndIf
		Next
	EndIf
End Function