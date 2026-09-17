# Runde 2, Teil 3: Eingabe, UI, Wirtschaft, Onboarding, Playtest

### N. Eingabe-Intents

Vorschlag der vollständigen Liste. Alles im Code heißt nach der Absicht, nicht nach der Taste.

| Intent | Desktop | Touch (später) | Anmerkung |
|---|---|---|---|
| `Interact` (Bauplatz, Geschütz, Boot anklicken) | Linksklick | Tippen | öffnet Radialmenü |
| `MoveBoat` | Linksklick auf Fluss | Tippen auf Fluss | Klick auf den Bunker = Heimfahrt, kein eigener Befehl |
| `FireTorpedo` | Space (halten = Salve) | Feuerknopf | nur angenommen, wenn Bug ausgerichtet |
| `ToggleDive` | **?** | Tauchknopf | siehe N1 |
| `StartWave` | Enter | Knopf am Bunker | |
| `ToggleOverlay` | Tab | Knopf | Toggle, gemerkt |
| `ToggleSpeed` | **?** | Knopf | 1× / 2× |
| `Pause` / `Back` | Esc | Knopf | schließt erst Menüs, dann Pausemenü |

- **N1: Taste für Tauchen.** Der PoC hat `T`. Die rechte Maustaste ist jetzt frei, seit das Radialmenü auf Linksklick liegt. **Rechtsklick = tauchen/auftauchen** hätte einen echten Vorteil: Die ganze Bootssteuerung liegt auf der Maus (links fahren, rechts tauchen), Space feuert. Die linke Hand macht nur Tab/Enter. Auf dem Trackpad ist Zwei-Finger-Klick okay. Oder lieber eine Taste (`T`, `Shift`)? Mein Vorschlag: Rechtsklick, `T` als Zweitbelegung.
  - Antwort: Rechtsklick ist OK
- **N2: Taste für 2× Geschwindigkeit.** `F` oder `+`? Und: Setzt die Geschwindigkeit sich beim Wellenstart zurück auf 1×? Ich empfehle **ja**, sonst startet man versehentlich Welle 4 auf 2× und verliert das Boot.
  - Antwort: ich bin mit deiner Empfählung einverstanden. Für 2x Geschwindigkeit bitte `F` nehmen.
- **N3: Klick auf das U-Boot.** Was passiert? Optionen:
  - a) nichts, Hover zeigt Infos
  - b) öffnet das Boots-Radialmenü (Reparatur, nächstes Modell, Ausrüstung)

  Und wenn b: Nur, wenn das Boot **am Bunker angedockt** ist (Werft-Fiktion), oder immer? Mein Vorschlag: Radialmenü nur angedockt, denn ein Modellwechsel mitten im Fluss wäre absurd, und die Werft am Bunker ist ein Grund heimzufahren. Das ist keine Blockade der Hand, sondern eine Regel mit klarer Fiktion. Während einer Welle angedockt: erlaubt?
  - Antwort: b, Radialmenü nur angedockt
- **N4: Halten von Space bei leeren Rohren.** Feedback: trockenes Klicken plus kurze Notiz "Rohre leer". Kein Ton-Spam bei Dauerhalten, nur einmal pro Druck. Okay?
  - Antwort: Okay

### O. Radialmenü-Inhalte

| Kontext | Einträge |
|---|---|
| Freier Bauplatz | 3 Blaupausen mit Kosten. Ghost zeigt Reichweite und dreht sich Richtung Fluss. Unbezahlbare sind grau. |
| Gebautes Geschütz | Upgrade (Ghost zeigt die *Änderung*: zweites Rohr erscheint), Verkaufen (Betrag sichtbar), Zielmodus |
| Angedocktes Boot | Reparieren (Kosten), nächstes Modell (Kosten, "ab Welle X") |
  - Antwort: perfekt, einverstanden
- **O1: Verkaufen ohne Bestätigungsdialog** (Säule 4 und dein "avoid confirmation dialogs"). Ein Fehlklick kostet aber 40 % des Preises. Genre-Lösung ohne Dialog: **Halten zum Verkaufen**, 0,5 s, ein Ring füllt sich. Loslassen vorher = Abbruch. Okay, oder Sofortverkauf?
  - Antwort: "**Halten zum Verkaufen**, 0,5 s, ein Ring füllt sich" ist super, einverstanden
- **O2: Zielmodus** als vierter Eintrag, der beim Klick durchschaltet (First → Strongest → …), oder als Untermenü mit vier Einträgen? Durchschalten ist ein Klick, Untermenü ist lesbar. Empfehlung: Durchschalten, mit Anzeige des aktuellen Modus als kleines Bleistiftsymbol am Geschütz im Overlay.
  - Antwort: durchschalten, Anzeige des aktuellen Modus im Radialmenü UND im Overlay des Geschützes ist OK
- **O3: Upgrades des Bootes.** Deine Tabelle hatte Modelle. Gibt es *zusätzlich* kleine Ausrüstungs-Upgrades (bessere Torpedos, stärkere Deckkanone), oder nur die drei Modelle? Für das MVP empfehle ich **nur Modelle**, weil drei Modelle mit Verhaltensänderung bereits drei Entscheidungen sind und die RM mit Geschützen konkurrieren. Ausrüstung kommt, wenn mehr Wellen kommen.
  - Antwort: deine Empfählung ist super, genau so machen wir es

### P. Tab-Overlay und Vorbereitungsphase

Overlay-Inhalte, Vorschlag: Reichweiten aller Geschütze, Ladebalken, Zielmodus, exakte HP, Boots-HP und Torpedos, Torpedo-Kanal des Bootes, Feindroute, Beschussreichweite der Zerstörer.

- **P1: Feindroute in der Vorbereitungsphase.** Neue Spieler müssen sehen, wo die Schiffe fahren, bevor sie bauen. Vorschlag: In der Vorbereitungsphase ist die Route **immer** als leichte Bleistift-Strichlinie sichtbar, im Kampf nur im Overlay. Okay?
  - Antwort: 
- **P2: Torpedo-Kanal des Bootes.** Auch ohne Overlay sichtbar, wenn man über das Boot hovert oder es gerade befehligt? Ich sage ja, denn "wo endet meine Gerade" ist die wichtigste Information des Spiels und darf nicht hinter Tab stecken.
  - Antwort: 
- **P3: Wellenvorschau.** Wie genau?
  - a) exakte Zusammensetzung als Symbole mit Zahlen (6× Schnellboot, 1× Zerstörer)
  - b) vage ("Zerstörer gesichtet")

  Empfehlung: a, als Funkspruch auf dem Papierzettel. Ein TD ohne genaue Vorschau bestraft Planung, und Planung ist das Spiel in der Vorbereitungsphase.
  - Antwort: 
- **P4: Hinweistext pro Welle** (der PoC hat "hint" pro Welle). Behalten als Teil des Onboardings, im Modus ohne Onboarding weg?
  - Antwort: 

### Q. Wirtschaft: das Gerüst, keine Zahlen

Die Zahlen kommen in Runde 3 mit einem Rechenmodell. Hier nur die *Regeln*, aus denen die Zahlen folgen.

- **Q1: Startkapital.** Dein Brainstorming-Gedanke war: fast pleite starten, damit die erste Versenkung vom Boot kommen muss. Ich finde das stark, weil es das Auffindbarkeits-Problem strukturell löst: Man *muss* das Boot benutzen, um überhaupt anzufangen. Drei Varianten:
  - a) Genug für eine Marineflak. Welle 1 ist auch ohne Boot schaffbar. Sicher, aber das Boot bleibt optional, und genau das ist im PoC passiert.
  - b) **Null oder fast null.** Welle 1 besteht aus 3 bis 4 langsamen leichten Booten auf einer langen Geraden. Torpedos oder Deckkanone töten sie. Nach Welle 1 hat man Geld für die erste Flak.
  - c) Genug für zwei Flak.

  Empfehlung b, mit einem Sicherheitsnetz gegen die Todesspirale: Die Deckkanone allein tötet Welle 1 auch dann, wenn alle Torpedos ins Ufer gehen. Der Spieler kann Welle 1 nicht verlieren, nur langsam gewinnen. Welche Variante?
  - Antwort: 
- **Q2: Belohnungszeitpunkt.** RM sofort beim Kill (Säule 2, sofortiges Feedback), keine Sammelaktion. Okay?
  - Antwort: 
- **Q3: Wellenabschluss-Bonus.** Ein kleiner fester Betrag am Ende jeder Welle, unabhängig von Kills. Grund: Anti-Todesspirale. Wer eine Welle schlecht spielt, hat trotzdem Einkommen für eine Korrektur. Ja oder nein?
  - Antwort: 
- **Q4: Die Zielgröße der Wirtschaft.** Das ist die eine Zahl, die alles bestimmt: **Wie viel Prozent von "alles gebaut und alles maximal" soll ein perfekter Durchlauf bezahlen können?** Im PoC war es über 100 %, deshalb war es zu leicht und Geld blieb liegen. Genre-Erfahrung: 50 bis 70 %. Damit muss man in jeder Runde entscheiden, was man *nicht* kauft. Mein Vorschlag: 60 %. Was sagt dein Bauchgefühl?
  - Antwort: 
- **Q5: Preisverhältnis Boot zu Geschützen.** Wenn ein "nur Boot"-Durchlauf möglich sein soll, muss das voll ausgebaute Boot ungefähr so viel kosten wie 3 bis 4 ausgebaute Geschütze. Ist das die Intention: Boot-Ausbau ≈ halbes Verteidigungsbudget?
  - Antwort: 
- **Q6: Verkaufen und Umbauen als Strategie.** Mit 60 % Rückerstattung kann man vor einer Zerstörerwelle Flak verkaufen und 15 cm bauen. Das ist gewollt (das Spiel belohnt Lesen der Vorschau)? Ich sage ja.
  - Antwort: 

### R. Onboarding-Skript, Wellen 1 bis 4

Prinzip: **kein Modal, kein Tutorial-Bildschirm.** Jeder Hinweis ist eine einzeilige Bleistift-Notiz am Ort des Geschehens, erscheint genau einmal, verschwindet nach der ersten richtigen Handlung. "Ohne Onboarding" schaltet nur diese Notizen ab, nichts anderes ändert sich.

| Welle | Lehrt | Hinweis (Ort) | Erfolgskriterium |
|---|---|---|---|
| 1 | Fahren, Feuern | Pfeil am Boot: "Klick auf den Fluss". Dann am Boot: "Space: Torpedo" | erster Torpedotreffer |
| 2 | Bauen | Ein Bauplatz pulsiert einmal: "Klick: Stellung bauen" | erste Flak steht |
| 3 | Konvoi, Upgrade, Vorschau | Am Zettel: "Funkspruch: nächste Welle". Am Geschütz: "Klick: ausbauen" | ein Upgrade gekauft |
| 4 | Tauchen | Zerstörer erscheint, Notiz am Boot: "Zerstörer feuert. Rechtsklick: tauchen" | erster Tauchgang unter Beschuss |

  - Antwort: 
- **R1:** Passt die Reihenfolge? Rammen und Zielmodi werden **nicht** gelehrt, die sind Entdeckung (Säule "Discovery" aus A4). Einverstanden, dass Rammen unerklärt bleibt und nur im Overlay als Hinweis existiert?
  - Antwort: 
- **R2:** Der Wellen-Hinweis aus dem PoC ("Kurven sind totes Wasser") ist Wissen, das man nicht selbst herausfindet. Soll das eine Onboarding-Notiz in Welle 2 sein (Torpedo läuft ins Ufer → "Torpedos laufen geradeaus. Kurven sind totes Wasser")? Das wäre eine **reaktive** Notiz: erscheint erst, wenn der Fehler passiert ist. Reaktive Hinweise sind die beste Form von Tutorial, weil sie nie zu früh kommen.
  - Antwort: 

### S. Playtest-Protokoll und Telemetrie

- **S1: Telemetrie im Greybox.** Ein lokales CSV pro Durchlauf, kein Upload: Welle erreicht, RM am Ende, RM ungenutzt, Geschütze pro Typ, Torpedos gefeuert und getroffen, Torpedos ins Ufer, Boot versenkt (wie oft, durch was), Zeit bis zum ersten Torpedo, Overlay an oder aus, Sitzungslänge. Okay? Das macht dein Video-Protokoll auswertbar, ohne 5 Stunden Video zu schauen.
  - Antwort: 
- **S2: Erfolgskriterien für den ersten Playtest** mit den 5 Freunden. Vorschlag:
  - 5 von 5 feuern in Welle 1 einen Torpedo ohne Hilfe
  - 4 von 5 bauen in Welle 2 ein Geschütz ohne Hilfe
  - 3 von 5 tauchen bei der ersten Zerstörer-Welle
  - mindestens 3 von 5 wollen einen zweiten Durchlauf (das ist der eigentliche Fun-Test)
  - niemand erreicht Welle 12 beim ersten Versuch (sonst zu leicht)

  Sind das die richtigen Schwellen?
  - Antwort: 
- **S3: Fragen nach der Sitzung**, offene Form: "Was wolltest du an Stelle X tun?", "Was hat das Boot gemacht, als du getaucht bist?", "Wofür hast du dein Geld ausgegeben und warum?". Keine "War es lustig?"-Fragen. Okay?
  - Antwort: 
