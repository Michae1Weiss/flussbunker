1. Das U-Boot ist das Spiel.
2. Zerstörung ist die Belohnung.
3. Die Hand des Spielers wird nie bestraft (eine manuelle Handlung, nie blockierte Interaktion, Zufall gibt nur).
4. Papier und Bleistift.

  - Antwort auf alle vier: alles richtig

---

# Runde 2, Teil 2: Geschütze, Gegner, Zerstörung

### K. Geschütze

- **K1: Anzahl und Lage der Bauplätze.** Kingdom Rush hat 6 bis 10 pro Karte. Vorschlag: 8 Bauplätze, davon etwa 5 an Kurven (dort wirken Geschütze, Torpedos nicht), 3 an Geraden, und **bewusst zu wenige, um alles abzudecken**. Knappheit erzwingt die Entscheidung. Einverstanden mit "zu wenige"?
  - Antwort: 8 ist OK. Die Zeit wird zeigen
- **K2: Radialmenü und laufendes Spiel.** Das Menü ist offen, die Welle läuft weiter, kein automatisches Pausieren. Sonst wird "Menü öffnen" zum versteckten Pause-Trick. Esc oder Klick daneben schließt. Okay?
  - Antwort: Okay, genau so habe ich es mir vorgestellt, ohne Pause, Wegklicken oder Esc schliesst.
- **K3: Bauzeit.** Der PoC hat eine Bauzeit. Drei Optionen:
  - a) sofort (Kingdom Rush)
  - b) kurze Bauzeit von 2 bis 3 s, das Geschütz ist im Bau sichtbar und schießt nicht
  - c) wie b, und auch **Upgrades** nehmen das Geschütz für 2 bis 3 s außer Betrieb

  Meine Empfehlung: b für den Bau (das ist "Stellung errichten"-Gefühl und ein echtes Risiko während einer Welle), Upgrades **sofort**. Sonst wird Upgraden während der Welle zur Falle, und das verstößt gegen Säule 3.
  - Antwort: b
- **K4: Verkaufen.** 60 % Rückerstattung, auch während der Welle? Und die Grenzfälle in der Tabelle unten.
  - Antwort: ja 60% passt
- **K5: Zielauswahl.** IDEAS.md wollte "Turm auf einen Schiffstyp ansetzen". Ich empfehle stattdessen die Genre-Standardmodi, weil man sie in einem Radialmenü mit vier Einträgen zeigen kann:
  - Vorderstes (Standard), Stärkstes, Schwächstes, Nächstes

  Typ-Priorität kann man später ergänzen. Einverstanden?
  - Antwort: einverstanden
- **K6: Treffen Granaten immer?** Im PoC verfolgen Granaten ihr Ziel und treffen garantiert. Nur Torpedos fliegen physikalisch gerade. Ich empfehle, das so zu lassen: Drehzeit und Nachladezeit sind der Unterschied zwischen den Geschützen, nicht Treffer oder Fehlschuss. Das langsame Drehen ist dann Darstellung plus eine kleine Verzögerung vor dem ersten Schuss.
  - Antwort: einverstanden
- **K7: Können Gegner Geschütze beschädigen?** Große Frage. Kingdom Rush: nein. Storm the House: ja. Ich empfehle **nein** fürs MVP: Geschütze sind permanent, das hält die Balance berechenbar. Wenn ja, brauchst du Reparatur, Reparaturkosten, Ziellogik der Schiffe und ein zweites Ding, das der Spieler beobachten muss.
  - Antwort: Storm the House auch nein, meiner Meinung nach. Ich gehe mit *nein*, Geschütze sind unverwundbar

**Grenzfälle Geschütze, bitte je "ok" oder anders:**

| Fall | Vorschlag |
|---|---|
| Geschütz wird verkauft, Granate ist in der Luft | Granate fliegt weiter und trifft. Sie ist eine eigene Entität. |
| Geschütz wird verkauft, Bauplatz ist wieder frei | Ja, sofort wieder bebaubar. |
| Radialmenü offen, Welle startet (Enter) | Enter wird angenommen, Menü bleibt offen. |
| Radialmenü offen, Spieler drückt Space | Torpedo feuert. Das Menü fängt keine Spiel-Intents ab. |
| Radialmenü offen, Spieler klickt auf den Fluss | Menü schließt, das ist gleichzeitig der Fahrbefehl? **Oder** Menü schließt nur, zweiter Klick fährt? Ich empfehle: **nur schließen**, sonst gibt es Fehlfahrten. |
| Trümmerstück landet auf einem freien Bauplatz | Rein optisch, Bauplatz bleibt bebaubar. Der Krater wird um das Geschütz herum gezeichnet. |
| Zu wenig RM, Spieler wählt Geschütz im Radialmenü | Blaupause bleibt ausgegraut, kurzes "zu teuer"-Feedback, Menü bleibt offen. |
| Zwei Ziele gleich weit | Deterministisch: das zuerst gespawnte. Kein Zufall. |
  - Antwort: das sind super durchdachte Sonderfälle an die ich nicht gedacht habe, sehr gut gemacht! Alle deine Vorschläge sind super, genau so wie du das sagst.
### L. Gegner

- **L1: Kollisionen zwischen Schiffen** (dein Bug in BUGS.md). Drei Optionen:
  - a) Abstand nur über Spawn-Abstand und gleiche Geschwindigkeit pro Gruppe. Mischwellen überlappen trotzdem.
  - b) **Konvoi-Regel:** Ein Schiff kann ein langsameres vor sich nicht überholen, es hält Mindestabstand. Simpel, lesbar, und erzeugt Konvois, also Explosionsziele.
  - c) Zwei Fahrspuren, kleine Boote überholen außen. Mehr Aufwand, mehr Lesbarkeit-Risiko.

  Empfehlung: b. Konsequenz für das Wellendesign: Schnelle Gruppen müssen *vor* langsamen kommen, sonst klumpt alles. Ein Monitor mit 16 Schnellbooten dahinter ist dann ein Klumpen, was für ein 15-cm-AoE ein Geschenk wäre.
  - Antwort: b als erstes. Ich habe mir auch c überlegt, aber nicht in MVP, vielleicht irgendwann können wir damit experimentieren aber nicht dieses Jahr glaube ich
- **L2: Wer greift das U-Boot an?** Nur Zerstörer (und der Monitor?). Schnellboote und Korvetten ignorieren es, außer durch Rammen. Das hält die Identität "Zerstörer = U-Boot-Jäger" scharf. Okay?
  - Antwort: okay
- **L3: Was sieht der Spieler vom Zustand eines Schiffs?** Greybox: HP-Striche unter beschädigten Booten wie im PoC. Später diegetisch: Rauch, Schlagseite, Feuer in drei Stufen. Für die Planung: Reicht "drei Schadensstufen sichtbar" als Ziel, oder willst du exakte HP im Tab-Overlay?
  - Antwort: im Tab-Overlay exakte HP bitte, sonst passt. Spricht etwas dagegen, HP im Tab-Overlay anzuzeigen?
- **L4: Reaktion auf Beschuss.** Weichen Schiffe aus, werden sie schneller, feuern sie zurück auf Geschütze (siehe K7)? Empfehlung: keine Reaktion. Schiffe sind Fahrpläne, das Spiel ist die Antwort darauf.
  - Antwort: einverstanden mit der Empfehlung 

### M. Zerstörung und Trümmer (der mechanische Teil)

- **M1: Sterbe-Matrix**, Klasse × tödliche Waffe. Das ist die Regel hinter deinen Immersion-Ideen. "Brennen" = driftet ans Ufer, keine Trümmer, kein Radius. "Explodieren" = sinkt an Ort und Stelle, 1 Trümmerstück (der Turm). "Zerbrechen" = zwei Hälften sinken, mehrere Trümmer, Schadensradius.

| | Marineflak | 8,8 cm | 15 cm | Torpedo | Rammstoß | Deckkanone |
|---|---|---|---|---|---|---|
| **leicht** | brennen | explodieren | **zerbrechen + Radius** | explodieren | kentern | brennen |
| **mittel** | brennen | explodieren | zerbrechen + Radius | zerbrechen + Radius | – | brennen |
| **schwer** | kann nicht töten | kann nicht töten | explodieren | zerbrechen + Radius | – | – |
| **Großkampfschiff** | – | – | – | **riesig zerbrechen + großer Radius + viele Trümmer** | – | – |

Passt die Matrix? Insbesondere: Soll ein Torpedo auf ein leichtes Boot nur "explodieren" oder auch zerbrechen? Ich habe es kleiner gehalten, damit das 15 cm gegen kleine Boote seinen spektakulären Sinn behält (deine Idee aus IDEAS.md).
  - Antwort: die Matrix passt, super Intuition!
- **M2: Kettenreaktionen.** Ein Radius tötet einen Nachbarn, der wieder explodiert. Erlaubt, ohne Obergrenze, weil nur Gegner betroffen sind? Ich sage ja, mit einer Darstellungsregel: Jede Kettenexplosion kommt 0,2 s nach der vorigen, damit man die Kette *sieht*. Sonst ist es ein Blitz und niemand versteht, was passiert ist.
  - Antwort: Ja super Idee mit Kettenexplosion, ich bin mit allem einverstanden!
- **M3: Trümmerarten.** Vorschlag: Turm (viel Schaden), Schornstein, Rettungsboot, Kisten, Anker, Luke. Schaden pro Art ist **fest**, nur der Landeort ist zufällig. Trümmer landen auf: Schiff (Schaden), Wasser (versinkt), Land (Krater). Nie auf Boot, Geschütz oder Bunker.
  - Antwort: einverstanden
- **M4: Trümmermenge pro Klasse.** leicht 1 bis 2, mittel 3 bis 4, schwer 5 bis 6, Großkampfschiff 8 oder mehr. Obergrenze gleichzeitig fliegender Trümmer für Lesbarkeit, etwa 20.
  - Antwort: einverstanden
- **M5: Was bringt Einsammeln in der Vorbereitungsphase?** Mit "nur RM" bleibt eigentlich nur **Bergung**: Trümmer auf Land sind RM wert. Zwei Wege:
  - a) Klicken zum Einsammeln, kleine Schnitzeljagd zwischen Wellen. Bewährter Mini-Loop (Sonnen in Plants vs Zombies), Risiko: nach 12 Wellen lästig.
  - b) Automatische Gutschrift am Wellenende, sichtbar als Transaktion "Bergung +37 RM" auf dem Papierzettel.
  - c) Einsammeln ganz streichen, Trümmer sind reine Darstellung; Übermäßiges Zerstören lohnt sich schon über den Radius.

  Ehrliche Meinung: b oder c. Der Motivations-Job "zerstöre spektakulär" wird schon vom Radius und der Kette erledigt. Bergung als Bonus-Zufall ist erlaubt (gibt nur), sollte aber klein bleiben, etwa 5 bis 10 % des Einkommens, sonst wird das Einkommen zur Lotterie.
  - Antwort: als erstes c, vielleicht die Idee fürs Später vorbehalten wenn noch neue Mechaniken dazukommen die dem Feature einen neuen Nutzen geben. Ich mag es nicht, in spielen wie Fallout Shelter Sachen zu sammeln wie in den Quests diese kleinen Kisten oder tote Creatures im Shelter um Kronkorken zu bekommen.
- **M6: Screen Shake.** Bei Torpedotreffern und schweren Explosionen, Stärke nach Klasse, abschaltbar in den Einstellungen (Barrierefreiheit). Okay?
  - Antwort: okay
