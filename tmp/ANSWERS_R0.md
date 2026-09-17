**R1: Die Torpedo-Mechanik und der "natürliche Fluss" schließen sich aus.**
Im PoC ist die gerade Strecke der Schusskanal. Die Form des Flusses *ist* die taktische Karte, und genau das war dein gutes Spielgefühl. BUGS.md will einen organischen Fluss, IDEAS.md scharfe Kurven und eine Mittellinie für das U-Boot. Ein geschwungener Fluss hat keine Geraden mehr. Dann brauchen Torpedos neue Regeln: Treffen sie das Ufer? Wird die Richtung an der Tangente bestimmt? Oder zielt man frei?
  - Fluss muss Geraden haben aber mit organischen Ecken und einem leicht gewölbten Ufer sodass das nicht wie ein Tetris aussieht (eng.). 

**R2: Das U-Boot fährt auf derselben Linie wie die Gegner.**
Wenn es aufgetaucht mitten im Fluss liegt, steht es jedem Schiff im Weg. Das ist genau deine Frage "Auftauchen unter einem Schiff", nur viel grundsätzlicher: **Ist das U-Boot ein Blocker?** Das wäre wie die Kaserne in Kingdom Rush, mit Rammschaden, Stau vor dem U-Boot und Kettenexplosionen im Stau. Diese eine Entscheidung verändert das ganze Spiel.
  - Antwort: unten beantwortet

**R3: Die Rolle des Torpedos geht verloren.**
DESIGN.md baut alles auf einer Regel auf: Panzerung stoppt Granaten, Torpedos ignorieren sie. IDEAS.md gibt dem 8,8 cm AP-Munition und macht das 15 cm zum "Panzerbrecher". Wenn Geschütze Panzerung knacken, braucht niemand mehr das U-Boot. Wir brauchen eine **Tabelle "Bedrohung → Antwort"**, bevor irgendeine Zahl feststeht.
  - Antwort: Eine Balance-Frage. Gerne würde ich zusammen mit dir die o.g. Tabelle erstellen

**R4: Das Granatwerfer-Loch.**
Der PoC hatte einen Mörser mit Flächenschaden gegen Schwärme. In IDEAS.md fehlt er. Die Marineflak ist "Anti-Swarm", schießt aber auf einzelne Ziele. Wer löst jetzt 20 Schnellboote?
  - Antwort: Mörser erstmal verwerfen, dafür ist Marinenflak da

**R5: Drei Arten, Geschütze zu platzieren, gleichzeitig im Text.**
- Der PoC nutzt ein Raster.
- IDEAS.md schreibt "freies Stück Land", also freie Platzierung.
- Dieselbe Datei schreibt "Show turret build spots only contextually", also feste Bauplätze.

Das sind drei verschiedene Spiele.

  - Antwort: siehe Antwort unten!

**R6: Die Zufallsideen widersprechen sich.**
Langsam drehende Türme, vorausberechnetes Zielen und Streuung (BUGS.md) machen das 15 cm fast nutzlos gegen schnelle Schnellboote. IDEAS.md will aber genau damit kleine Boote spektakulär sprengen. Dazu kommt: Zufällige Trümmer machen zufälligen Schaden. TD-Spieler planen, und Zufall *im Ergebnis* fühlt sich unfair an, Zufall *in der Darstellung* dagegen gut. Diese Unterscheidung stammt aus Greg Costikyans Buch *Uncertainty in Games*.
  - Antwort: Zufall *im Ergebnis* fühlt sich unfair an, Zufall *in der Darstellung* kann ich nicht verstehen, elaborate please und sage wo ich nachlesen kann über dieses Prinzip? Langsam drehende Türme ist eine Übertreibung meinerseits, sorry, die müssen schon schnell genug sein um die schnellen Boote abzufeuern und Streuung steht unter Frage - siehe unten. Da brauche ich Tipps und Anregungen und Meinung oder alternative Wege

**R7: Sinkende Schiffe und automatisches Zielen.**
"Ein zweiter Treffer auf ein sinkendes Boot → riesige Explosion". Wenn sinkende Schiffe Ziele bleiben, verschwenden Türme Schüsse an tote Schiffe (Overkill-Problem). Wenn nicht, passiert der coole Moment nur durch Zufall.
  - Antwort: meine Idee mit dem Treffer in das sinkende Boot finde ich jetzt komisch und nicht brauchbar, bitte erstmal verwerfen!

**R8: Drei bis vier Fortschrittswährungen.**
RM, XP, Trümmerteile und dazu die Modell-Upgrades des U-Boots, die Level *und* RM kosten. Das ist viel Kopflast für ein minimalistisches UI.
  - Antwort: siehe unten. Ich tendiere dazu, nur RM zu haben

**R9: Minimales UI trifft auf nachgewiesen schlechte Auffindbarkeit.**
Die Kommentare in `uboat.rs` halten eine Playtest-Beobachtung fest: Du hast drei Runden gespielt, ohne das zweite U-Boot zu entdecken. Ein verstecktes UI macht dieses Problem größer. Onboarding muss als eigenes System geplant werden.
  - Antwort: Onboarding ist geplannt. Beim Spielstart will ich zwei Optionen zeigen: mit onboarding oder ohne.

**R10: Tastenbelegung und Ergonomie.**
- Leertaste startet im PoC die Welle, in IDEAS.md feuert sie Torpedos, und Enter startet die Welle.
- Tab steht im Fließtext als Umschalter, in der Liste als Gedrückt-Halten.
- Rechtsklick gedrückt halten, während man links das U-Boot steuert und Tab hält: Das geht mit Maus gut, mit einem Mac-Trackpad schlecht, mit Steam Deck oder Touch gar nicht.
  - Antwort: seht guter Punkt! "Rechtsklick gedrückt halten" finde ich mittlerweile schlecht. Space feuert torpedos enter startet welle. Tab muss als toggle sein, idee mit Gedrückt halten stammt von GPT und ich finde die schlechter als toggle, was sagst du? Hast du eine starke Meinung dazu, Beispiele aus anderen spielen?

**R11: Das Thema. Das sage ich als Berater deutlich.**
"Stil: zweiter Weltkrieg, Nazi" plus Propaganda-Ästhetik plus "bisschen lustig" plus der Name eines echten Orts. Bunker Valentin ist eine Gedenkstätte, gebaut mit Zwangsarbeit, über tausend Menschen sind dort gestorben. Das ist eine **Produktentscheidung mit rechtlichen Folgen**:
- **§86a StGB:** Hakenkreuze in Spielen sind in Deutschland nur mit USK-Prüfung und Sozialadäquanz-Einzelfallentscheidung erlaubt.
- **Store-Richtlinien und Presse:** Die Aufnahme hängt am Ton.
- Dass man die "Täterseite" spielt, ist legitim, siehe Silent Hunter. Humor plus Täterseite plus echter Gedenkort ist aber die riskanteste Kombination.

Das schließt nichts aus. Es muss nur *bewusst* entschieden werden, bevor Assets entstehen.

  - Antwort: siehe meine Antwort unten. Kurz gesagt: keine Nazi-Symbolik, keine direkten Bezüge auf "Bunker Valentin", komplizierte Themen und Symbolik raus, fiktiv nehmen, unproblematische Sachen wie Uboot klassen und Waffen echt nehmen - wenn das natürlich OK ist

**R12: Veraltete Doku und Code, der deiner Kommentar-Philosophie widerspricht.**
- DESIGN.md und ARCHITECTURE.md sind KI-Texte. Die README behauptet noch "Not compiled".
- Die Doku passt nicht mehr zu IDEAS.md.
- Im Code bestehen etwa 536 von 3.400 Zeilen `src/` nur aus Kommentaren, dazu ganze Essays als Moduldoku.
- Lokal heißt der Branch `master`, dein Git-Kontext nennt aber `main` als Hauptbranch. Ein Remote gibt es nicht.
  - Antworten sind unten!

**R13: Umfang.**
"Soll sehr lange und sehr viel wachsen" ist der häufigste Grund, warum Hobbyspiele nie erscheinen. Das Gegenmittel sind **Design-Säulen** als Filter und **Meilensteine mit klaren Abbruchkriterien**, nicht weniger Ideen.
  - Antwort: Design-Säulen und Meilensteine finde ich super!

**R14: Validierung.**
Fünf Runden durch dich selbst sind betriebsblind. Die Faustregel nach Jakob Nielsen: Fünf fremde Tester finden etwa 80 % der Usability-Probleme. Ein Tipp dazu: Bevy kann nach WebAssembly bauen. Ein Link auf itch.io senkt die Hürde für Tester massiv.
  - Antworten: ich werde mein Spiel wenn es soweit ist kompilieren und bei meinen 5 Freunden testen, ich gebe das Spiel, nehme die Session auf Video auf und sage dabei nichts kommentiere nichts und dann nachher kann ich alles analysieren. Der Spielende kann dabei sprechen und kommentieren, ich nicht!

---

# 3. Fragen, Runde 1: das Fundament

Alles Spätere hängt an diesen Antworten, deshalb kommen die Detailfragen erst in der nächsten Runde. Du kannst kurz antworten, z. B. "A3: b". Wo ich eine Empfehlung habe, steht sie in Klammern.

### A. Vision und Ziel
- **A1:** Was ist das Projekt für dich?
  - a) Lernprojekt, das "fertig genug" sein soll
  - b) kostenloses Release (itch.io)
  - c) kommerzielles Release (Steam)
  - d) Portfolio
    - Antwort: Lernprojekt, das "fertig genug" sein soll und auch als Portfolio
- **A2:** Wie viele Stunden pro Woche hast du realistisch? Gibt es eine Deadline oder ein Wunschdatum für eine erste öffentliche Version?
  - Bis Ende des Jares, pro Woche habe ich 2-3 Abende je 3-4 Stunden
- **A3:** Arbeitest du allein? Wer zeichnet die Assets, du selbst? Wer macht Sound?
  - Ich arbeite allein, mit KI-Unterstützung. Assets werde ich vom KI generieren lassen als MVP und später wenn ich zu viel Zeit habe selber abmalen und korrigieren damit die echt sind.
- **A4:** Welches Gefühl soll ein Spieler nach 30 Minuten haben? Wähle maximal 2 aus dem MDA-Framework (Hunicke, LeBlanc, Zubek): Herausforderung, Fantasie ("Ich bin Kommandant"), Entdeckung, Ausdruck (eigener Build), Spektakel und Chaos, Meisterschaft.
  - Spektakel und Chaos, Herausforderung, Entdeckung (unterschiedliche Wellen, neue Gegner, neue Upgrades).
- **A5:** Nenne 3 Spiele als Referenz und je *das eine Ding*, das du daraus willst. Ich vermute Kingdom Rush, Bodycam, Silent Hunter. Stimmt das?
  - Silent Hunter mag ich das Gefühl des Abfeuerns von Torpedos und das Ticken des Timers und Booom mit Wassersäule. Ich liebe auch Tower Defense Spiele wie toy defense 2 mit Upgrades, Storm the House 3 für passive Attaken und möglichkeit proaktiv mit Waffe gegner abschiessen.
- **A6:** Nenne 3 Dinge, die das Spiel ausdrücklich **nicht** sein soll, z. B. "keine Simulation", "kein Idle-TD", "kein Mikromanagement-Stress".
  - Keine RTS, keine Idle-TD, Mikromanagement-Stress - bitte NICHT, kein Realismus, kein Spiel wo man eine Session 2 Wochen spielt, eine Session ist ein Spielvorgang, 30 Minuten oder so

### B. Aufbau eines Durchlaufs
- **B1:** Welche Struktur?
  - a) eine Karte, 12 Wellen, fertig
  - b) Kampagne mit mehreren Karten
  - c) Roguelite mit Fortschritt über mehrere Durchläufe hinweg
  - d) Endlosmodus
  - e) eine Kombination
    - Antwort: für MVP eine Karte und fixe Wellen - 12 ist gut for starters, später deutlich mehr Wellen und mehr Upgrades
- **B2:** Gewünschte Länge einer Sitzung oder eines Durchlaufs?
  - Antwort: MVP 10-15 Minuten, später mit mehr Content bis zu 30 Minuten oder länger
- **B3:** Bleibt irgendetwas zwischen Durchläufen erhalten (Trümmer-Perks, freigeschaltete U-Boot-Modelle), oder ist jeder Durchlauf ein Neustart?
  - Jeder Durchlauf sit ein Neustart
- **B4:** Wie verliert man? Bunker-Integrität wie im PoC, oder etwas anderes? Und wie gewinnt man?
  - Bunker-Integrität. Gewinnt man wenn man alle Wellen durchgehalten hat
- **B5:** Soll es während des Kampfs eine Pause oder doppelte Geschwindigkeit geben? (Empfehlung: ja, beides ist Genre-Standard.) Darf man während einer Welle bauen?
  - Alles ja.

### C. Die drei offenen Kernentscheidungen
- **C1: Wie platziert man Geschütze?**
  - a) Raster wie im PoC
  - b) frei, mit Kollisionsprüfung
  - c) feste Bauplätze wie in Kingdom Rush
    - Antwort: feste Bauplätze

  (Empfehlung: c. Das passt am besten zu "Stellung bedienen", zum Radialmenü, zu "Bauplätze nur bei Bedarf zeigen" und zur Balance.)
- **C2: Welche Form hat der Fluss?**
  - a) eckiges Raster
  - b) organischer Spline
  - c) organisch aussehend, aber mit bewusst gebauten Geraden als Schusskanälen
    - Antwort: c
- **C3: Wie feuert das U-Boot?**
  - a) automatisch, sobald ein Ziel im Kanal ist (PoC)
  - b) manuell mit der Leertaste (IDEAS)
  - c) manuell, aber mit Salve und kurzem Zielen
    - Antwort: b und wenn mit salve dann einfach mehrmals drücken oder halten. Frage wie das mit Touch funktionieren soll... Abfeuern macht ja Spass! Und eventuell kann man automatisches Abfeuert ja aktivieren mit dem Nachteil dass die langsamer und nicht instant abgefeuert werden und mit einem sleep() zwischen Schüssen und mit manuellem Modus kann man fast alle auf einmal abfeuern und abhauen. ist das gamedesign-technisch gute idee oder design smell? sei echt zu mir. ich schätze deine Meinung über die Aufmerksamkeit!

  Hinweis: Bei b oder c wird aus dem TD ein halbes Actionspiel. Das ist legitim, aber dann geht deine Aufmerksamkeit fast ganz aufs U-Boot, und die Türme werden zur Kulisse. Soll das so sein?

### D. Das U-Boot im Fluss (siehe R2)
- **D1:** Kann ein aufgetauchtes U-Boot mit Schiffen kollidieren, also Schiffe **blockieren**? Und getaucht: Fahren Schiffe einfach darüber hinweg?
  - Antwort: Schiffe rammen das Uboot und und das Uboot verliert HP und Schiff steht vor dem Uboot und macht damage die ganze zeit bis das uboot sinkt. Schiff verliert ebenso HP aber weniger als uboot, kommt auf das Uboot an. 
- **D2:** **Deine Frage: Was passiert, wenn das U-Boot direkt unter einem Schiff auftaucht?** Die Optionen, damit du siehst, wie unterschiedlich sie sich spielen:
  - a) **Blockiert:** Auftauchen geht nicht, es gibt Feedback "Kontakt über uns", und der Befehl bleibt nicht in der Warteschlange.
  - b) **Verzögert:** Auftauchen startet automatisch, sobald die Stelle frei ist.
  - c) **Rammstoß als Mechanik:** Beide nehmen Schaden. Kleine Boote kentern, bei großen wird das U-Boot schwer beschädigt. Das wäre eine eigene Fähigkeit mit Risiko.
  - d) **Wegdrücken:** Physik schiebt die beiden auseinander (Empfehlung: nein, das ist unlesbar und schwer abzustimmen).
    - Antwort: c! Rammstoß als Mechanik finde ich FASZINIEREND! Boot kentert ist geil. Bisschen Mikromanagement falls keine Torpedos und man vor dem Bunkter steht und das Boot versenken muss aber rammen kommt nicht in frage - vllt so machen dass Rammstoß effizienter und mehr dem Gegner wehtut als das Rammen im aufgetauchten Zustand? Brauche deine ehrliche Einschätzung. Bei großen Schiffen würde ich auch sagen - option a - blockieren, dies ist nicht frustrierend (wenn man so sein Uboot verliert löscht man das Spiel!).

  Ich tendiere zu c für kleine Boote und a für große. Welche Richtung reizt dich?
- **D3:** In welche Richtung zeigt das U-Boot, und in welche Richtung feuert es? Muss es wenden, um stromabwärts zu schießen? In deiner Tabelle steht "4+1" Rohre. Soll das eine Heckrohr eine Mechanik werden, also Schießen nach hinten?
  - Ich würde sagen nur nach vorne schiessen. Heckrohr-Mechanik ist interessant, aber ich weiß nicht ... somehow overkill und komisch.
- **D4:** Wenn das U-Boot zerstört wird: Was geht verloren (XP, Level, gekauftes Modell)? Was kostet der Wiederaufbau (Geld, Zeit, eine Welle aussetzen)? Das Gefühl, die Einheit beschützen zu wollen, kämpft hier gegen Frust.
  - Hier muss ich um deinen Rat bitten... Alles zu verlieren wäre richtig frustrierend. Geld für die Aufbau zu verschwenden ebenso obwohl wenn man gerammt hat hat man ja auch was absichtlich geopfert, nicht? Vllt bis Ende der Welle Uboot im Fluss liegen lassen und zwischen Wellen dann die Bau anfordern kann und 10 Sekunden warten auf den Bau? Oder ist das zu kleiner Preis? Oder man zahlt geld und wenn kein geld kommt das uboot beschädigt raus und die Reparatur kostet Geld und beschädigtes Uboot ist schwacher - weniger torpedos weniger damage.
- **D5:** Gibt es jemals mehr als ein U-Boot?
  - Ich glaube nicht. Ein Uboot ist bereits mit den ganzen neuen Mechaniks ein Objekt dass richtig anspruchsvoll ist, im Sinne von ... User muss dem Uboot viel Aufmerksamkeit schenken, zwei Uboote können einen Überfordern oder die müssen dann Automatisch handeln.

### E. Zerstörung, Zufall und Belohnung
- **E1:** Gilt die Regel **"Zufall in der Darstellung ja, Zufall im Ergebnis nur begrenzt und lesbar"**? Konkret: Dürfen Trümmer *zufällig* über Schaden an anderen Schiffen entscheiden?
  - Antwort: ich würde sagen ja, entweder schaden oder am Land landen oder wenn keine Schiffe ins Wasser oder ans Land. Ich denke dabei an Hearthstone mit Random-Ereignissen die manchmal eine echte Dophamin-Bombe sind. Oder ist dies nicht der Fall? Ehrliche Meinung bitte
- **E2:** Streuung und Fehlschüsse der Türme: echter Spielwert oder nur Optik? Können eigene Granaten das **eigene aufgetauchte U-Boot** treffen? Friendly Fire wäre spannend, aber potenziell ärgerlich.
  - Antwort: kein Friendly Fire. Streuung und Fehlschüsse erstmal auf die lange Bank schieben, kann Spieler frustrieren. Ich weiß nicht wie man dies als Spielwert implementieren kann. Ideen?
- **E3:** Ein brennendes Schiff: Fährt es weiter, kann es noch den Bunker erreichen, schießt es noch? Wann gibt es die Belohnung, beim tödlichen Treffer oder erst beim Untergang?
  - Ein brennendes Schiff driften in den Ufer ab und strandet da ohne Weg anderen zu blockieren. Was genkst du? Ideen?
- **E4:** Trümmerteile einsammeln: Wer sammelt und wann?
  - a) automatisch
  - b) per Mausklick mitten im Kampf (kostet Aufmerksamkeit)
  - c) das U-Boot fährt drüber
  - d) Aufsammeln in der Vorbereitungsphase
    - Antwort: Aufsammeln in der Vorbereitungsphase
- **E5:** Wofür sind Trümmer da, als *eigene* Währung neben RM? Oder kann man sie weglassen, damit weniger Währungen im Spiel sind (siehe R8)?
  - Nach deiner Anmerkung über XP und RM und Trümmerwährung habe ich mir überlegt ... vielleicht alles abschaffen und nur bei RM bleiben? Dann kann man ja das ganze Geld in Uboot investieren und somit andere Strategie benutzen - no-turrents, als eigener Spielstill. Weil im aktuellen Spiel habe ich in Late Game oft Uboot gesteuert, bereits grosse Turrets gehabt und versucht Uboot mit Xp hochzuleveln und am ende gemerkt wie viel Geld ich eingesammelt habe und nicht benutzt habe stattdessen habe ich XP des Uboots gefarmt. Und was noch schlimmer war - die abgeschossene Kleinbote die Turrets gekillt haben haben mir meine XP weggenommen, und ich habe 2 Torpedos in Destroyer investiret und das Turret hat den gekillt und ich sass ohne profit da - fühlte mich verarscht an!
- **E6:** Versenkte Wracks unter Wasser: nur Optik, oder Hindernis für Torpedos und U-Boot? Wird der Fluss mit der Zeit enger?
  - nur Optik

### F. Thema und Ton (siehe R11)
- **F1:** Was genau heißt "Nazi-Stil"? Kriegsmarine-Ästhetik (Balkenkreuz, Uniformen, Schrifttypen)? Oder auch NS-Symbole?
  - ich bin doch gegen Nazi-Stil, ich will nicht NZ direkt ansprechen da, keine Namen nennen, nur Optik der Zeit. Kriegsmarine-Ästhetik ja, NS-Symbole nein.
- **F2:** Ton?
  - a) ernst und neutral wie Silent Hunter
  - b) Satire oder Groteske wie Wolfenstein
  - c) verspielt und cartoonhaft
    - Antwort: c

  "Bisschen lustig" plus Täterseite ist die heikelste Mischung, deshalb frage ich.
- **F3:** Bleibt der echte Name "Bunker Valentin", oder wird der Ort fiktiv? DESIGN.md schlägt schon einen historischen Hinweis vor.
  - nein keine echten Namen wie "Bunker Valentin", etwas Fiktives am besten so wenig wie möglich namen und wenig story.
- **F4:** Zielmärkte? Deutschland gehört sicher dazu, und das entscheidet über die USK-Frage.
  - Freunde, Deutschland

### G. Technik und Konventionen
- **G1: Zielgeräte?** Windows, macOS, Linux, Steam Deck, Web (WASM, ideal für Playtests), Mobile oder Touch. Davon hängen ab: das Radialmenü per Rechtsklick, Tab gedrückt halten, und dass die Karte ohne Kamerafahrt auskommt.
  - Desktop-First, dann Web, dann iPad, dann Handys.
- **G2:** Welche Seitenverhältnisse und Mindestauflösung? Die Karte ist fest und die Kamera steht still, also: Was passiert bei 16:10 (Mac, Deck), 21:9, 4:3?
  - 16:9, ScalingMode::AutoMin
- **G3: Sprachen:**
  - a) UI-Sprache im Spiel (deutsche Einheitennamen als Flair, UI aber übersetzbar?)
  - b) README
  - c) Code-Bezeichner (vermutlich Englisch)
  - d) Code-Kommentare
  - e) Commit-Messages
  - f) Design-Doku
    - Antwort: Alles auf Englisch bitte, deutsche Namen nur in Sachen wie Einheitennamen.

  Meine Empfehlung: alles im Repo auf Englisch, Design-Doku darf Deutsch sein, Spieltexte von Anfang an übersetzbar anlegen.
- **G4: Commit-Konvention:** Conventional Commits? Dein letzter Commit `docs: write ideas out` sieht schon danach aus. Mit Scopes wie `feat(uboat): ...`?
  - Conventional Commits bitte
- **G5: Git:** Wird der Hauptbranch `main` oder `master`? Feature-Branches oder direkt auf den Hauptbranch? Kommt ein Remote auf GitHub, Codeberg oder woanders? CI mit `fmt`, `clippy` und `test`?
  - main. Alles auf den Hauptbranch. GitHub. CI später, bitte als TODO aufnehmen mit Ideen.
- **G6:** Wo landen Bugs und Ideen: weiter in Markdown-Dateien, oder als Issues im Tracker?
  - Was ist da die Best Practice? Ich werde ja Github benutzen. Vllt da im Board (Trello-Mässig) oder was würdest du vorschlagen?
- **G7: Bevy-Updates:** Bevy bricht die API etwa alle 3 bis 4 Monate. Sofort mitziehen, oder bei einer Version bleiben bis zu einem Meilenstein?
  - bei einer Version bleiben bis zu einem Meilenstein oder bis ich migrieren will, DU und KI entscheidest das nicht
- **G8: Kommentare:** Du willst das separat klären, das nehme ich so an. Nur eine Frage vorab: Sollen die bestehenden Essay-Kommentare beim nächsten Umbau radikal entfernt werden, oder schreiben wir erst die Konvention?
  - Erst Konvention schreiben erst dann radikal entfernen. Ohne Konvention weiß du nicht, was und wie du entfernen sollst.
- **G9:** Die bestehende Doku (DESIGN.md, ARCHITECTURE.md): wegwerfen und neu schreiben, oder überarbeiten?
  - Nur das was wichtig und hilfreich behalten, altlast wegwerfen (ist ja alles in git, nicht gefärhlich!)
- **G10:** Ist der PoC-Code eine Basis zum Weiterbauen, oder wird er nach dem Greyboxing neu aufgesetzt? ("Design it twice", siehe Ousterhout.)
  - Diese PoC-Code ist als Beispiel eines funktionierenden Spiels und ein Beispiel, wie man's machen könnte. Idee war, kleine Features und Mechaniken gekapselt in examples Ordner zu bauen damit man die Live testen kann, kleine Lego-Bausteine sozusagen. Wie die End-Architektur GENAU aussehen soll ist eine offene Frage aber aktuelle Aufbau ist meiner Meinung nach OK und kann als Basis benutzt werden, zusammen mit bevy-skill mit Best practices was Architektur angeht. So wie das Ousterhols gesagt hat - design ist laufendes Ding und am Anfang kann man kein perfektes Design machen.
