Fragen:
- **H1:** Das 8,8-AP-Upgrade *halbiert* die Panzerung, hebt sie aber nicht auf. Damit bleibt der Zerstörer Torpedo-Territorium. Einverstanden? Oder soll ein voll ausgebautes 15 cm einen Zerstörer allein töten können (langsam, teuer)?
  - Antwort: erstmal soll das bei deiner Tabelle verbleiben, finde ich so sehr gut balanciert.
- **H2:** Die Konsequenz: Ein Durchlauf **ohne U-Boot ist unmöglich** (schwere Schiffe brauchen Torpedos), ein Durchlauf ohne Geschütze ist *möglich*. Ist das eine bewusste Design-Säule ("Das U-Boot ist die Identität des Spiels")? Ich empfehle ja, aber es heißt: Wer das Boot nicht mag, verliert in Welle 4.
  - Antwort: Ein Durchlauf **ohne U-Boot ist unmöglich** ist richtig so, ja! Wer das Boot nicht mag, verliert in Welle 4. Genau so wie in Storm the house.
- **H3:** Gibt es einen Gegner, der das U-Boot *speziell* bestraft, wenn es aufgetaucht ist? Im PoC werfen Zerstörer Wasserbomben auf ein *aufgetauchtes* Boot. Das ist fiktional falsch (Wasserbomben sind gegen getauchte Boote). Vorschlag: Zerstörer **beschießen** aufgetauchte Boote, getaucht ist man sicher. Gleiche Mechanik, richtige Fiktion. Okay?
  - Antwort: Ja gerne Zerstörer U-Boote beschiessen lassen.
- **H4:** Brauchen wir für 12 Wellen einen sechsten Gegnertyp? Meine Meinung: nein. Fünf Typen, davon jeder mit einer klaren Antwort, reichen. Variation kommt aus der Mischung.
  - Antwort: Fünf Typen reichen erstmal!

### I. U-Boot-Regeln

- **I1: Feuern nur aufgetaucht?** Der PoC sagt ja. Das ist historisch falsch, aber spielerisch die Kernspannung: aufgetaucht = gefährlich + kann schießen, getaucht = sicher + blind. Ich empfehle, das zu behalten. Damit fällt der "Unterwasserjäger" (Typ XXI) aus deiner Tabelle weg, oder er wird zum Sonderfall mit eigenem Nachteil.
  - Antwort: bitte die PoC-Mechanik im Kern behalten. Das ist auch so in Superme Commander 2 mit Kraken. Siehe https://supcom2.fandom.com/wiki/Kraken_Experimental_Giant_Submarine. Unterwasserjäger soll aber eine Ausnahme sein, dafür kann er aber nicht rammen und hat kein Geschütz.
- **I2: Nachladen nur am Bunker?** Der PoC sagt ja, das erzeugt den Rhythmus Ausfahrt → Schießen → Heimfahrt. Dein Ritterkreuz-Frust kam vom langsamen Rohr-Nachladen, nicht von der Heimfahrt. Vorschlag: Rohre laden schnell (Salve möglich), Nachschub nur am Bunker. Behalten?
  - Antwort: ja dein Vorschlag ist super
- **I3: Torpedo am Ende der Geraden.** Der Torpedo läuft geradeaus bis zur nächsten Kurve und schlägt dort ins Ufer (Wassersäule, kein Schaden). Kurven sind totes Wasser, wie im PoC. Einverstanden, oder sollen Torpedos der Flussmitte folgen können?
  - Antwort: ich fand das OK dass Kurven totes Wasser sind, ich muss somit uboot immer wieder umstationieren und auf neue Position bringen, ich konnte so nicht von 1000M abfeuern wie eine Ratte und schnell nachladen sondern musste aktiv umpositionieren. Kurven Totes Wasser sind für mich OK solange das aus deiner Sicht kein Design oder Gameplay Smell ist.
- **I4: Ausrichtung.** Das Boot zeigt in Ruhe immer stromaufwärts, zur See, also dorthin, wo Gegner herkommen. Flussabwärts fährt es rückwärts, langsamer. Dann ist "nur nach vorne feuern" immer sinnvoll und der Rotations-Bug aus BUGS.md verschwindet als Regel. Oder soll es wenden, mit Wendezeit als Kosten?
  - Antwort: ich würde sagen wenden ist OK, uboot das rückwärts fährt ist somehow komisch, nicht? Oder denkst du das wäre für Gameplay besser? Bitte ehrlich
- **I5: Deckkanone.** Automatisch feuern wie ein Geschütz, nur wenn aufgetaucht? Empfehlung: ja. Das Boot hat *eine* manuelle Handlung (Torpedos) und *eine* automatische (Kanone). Zwei manuelle Waffen wären der Mikromanagement-Stress, den du nicht willst.
  - Antwort: ja bitte Geschütz automatisch feuern
- **I6: Bewegung.** Klick auf den Fluss, Boot fährt zum nächsten Punkt auf der Mittellinie. Darf man während der Fahrt tauchen? Während des Tauchvorgangs den Befehl abbrechen? (PoC: Befehle während des Übergangs werden ignoriert.)
  - Antwort: Ja darf man. Das würde ich als sehr gutes UI/UX empfinden. Blockiertes UI und blockierte Interaktion ist frustrierend. Wenn man missklickt muss man nicht dafür büssen und warten bis fertig sondern schnell reagieren können und das machen was man beabsichtigt hat.
- **I7: Im Bunker angedockt.** Ist das Boot dort unverwundbar, auch wenn Schiffe den Bunker erreichen? Empfehlung: ja, der Bunker ist der sichere Hafen, sonst gibt es einen Todesspiral-Fall beim Nachladen.
  - Antwort: ich sage ja zu deiner Empfählung. Uboot im Bunker ist unverwundbar
- **I8: Der Bunkerdurchbruch selbst.** Was macht ein Schiff, das den Bunker erreicht? Im PoC verschwindet es und zieht Integrität ab. Bleibt das so, oder feuert es auf den Bunker und man kann es noch töten?
  - Antwort: PoC behalten, kann ja später auch angepasst werden (KA was der Vorteil vom Feuern auf Bunkter ist, warum fragst du? Ist das feuern besser?) aber erstmal bei PoC
- **I9: U-Boot-Modelle.** Deine Tabelle hat 7 Modelle in zwei Zweigen. Für 12 Wellen sind das zu viele, um jedes zu erleben. Vorschlag MVP: **3 Modelle linear** (VIIA → VIIC → IXC), Verzweigung kommt mit mehr Wellen. Jedes Modell muss das *Verhalten* ändern, nicht nur Zahlen: z. B. IXC hat 2 Salven, aber taucht langsamer. Okay?
  - Antwort: Okay
- **I10: Sichtbarer Preis des Tauchens.** 1,4 s Übergang war die einzige Kosten im PoC. Reicht das, oder soll getaucht auch die Fahrt merklich langsamer sein (PoC: 70 %)? Beides zusammen ist mein Vorschlag.
  - Antwort: dein Vorschlag

### J. Zwei Grundsatzfragen, die ich sonst später bereue

- **J1: Arbeitstitel.** Brauchen wir jetzt einen, damit Crate, Repo und Docs einen Namen haben? Vorschläge, alle fiktiv: *Wesermündung* (zu real?), *Flussbunker*, *Sperrfeuer*, *Der Kanal*. Oder du hast schon einen.
  - Antwort: *Flussbunker* finde ich ok
- **J2: Entscheidungen festhalten.** Ich würde die bisher festgezurrten Entscheidungen jetzt in `docs/planning/decisions.md` schreiben (Englisch), damit nichts in dieser langen Unterhaltung verloren geht. Das ist Doku, kein Code. Soll ich, oder wartest du lieber, bis alle drei Teile durch sind und ich alles auf einmal in die Endstruktur gieße?
  - Antwort: kannst du gerne anfangen da zu schreiben

Antworte wieder kurz, "H1: ja, I4: wenden" reicht. Wo du unsicher bist, sag es, dann lege ich Optionen vor.
