Ich will das minimalistische diegetische UI haben.
Style ist: zweiter Weltkrieg, Nazi

## Turret bauen
Der Spieler bewegt die Maus auf ein freies Stück Land.
Rechtsklick halten → Radial Menu erscheint direkt am Cursor.
Aber die drei Optionen wären keine normalen UI-Icons.
Stattdessen erscheinen dort kleine, halbtransparente Turret-Blaupausen
Sobald du die Maus beispielsweise Richtung 88-mm-Geschütz ziehst:
Blueprint wird heller
Geschütz dreht sich bereits in die zukünftige Ausrichtung
Platzierungsradius erscheint eventuell als sehr dezente Linie
Kosten erscheinen direkt neben dem Blueprint
loslassen → bauen
Das fühlt sich viel stärker nach "Ich bediene eine Stellung" an als nach "Ich klicke einen Tower-Defense-Button".
Ich würde die drei Türme nicht als reine Kaliberleiter bauen, sondern als drei klar getrennte Rollen. Jeder Upgrade-Schritt sollte das Spielverhalten sichtbar verändern, nicht nur +20 % Damage geben.
| Turret                    | Rolle                      | Stufe 1              | Upgrade 1                        | Upgrade 2                    |
| ------------------------- | -------------------------- | -------------------- | -------------------------------- | ---------------------------- |
| **Marineflak**            | Anti-Swarm / leichte Boote | 1 schnelles Geschütz | höhere Kadenz + größeres Magazin | 2 längere Rohre + Autoloader |
| **8,8 cm Küstengeschütz** | Präziser Allrounder        | Einzelgeschütz       | bessere Feuerleitung + AP        | Doppelgeschütz / Burst       |
| **15 cm Batterie**        | Heavy / Panzerbrecher      | 1 Rohr               | 2 Rohre                          | 3 Rohre                      |

## Geld
Rechts oben ein kleines Stück Papier auf dem Tisch bzw. am Bunker
Da stehen wie viele RM user hat

## Grafik

## Immersion
Wenn ein Schiff zerstört wird fliegt der Turm in eine random Richtung entweder ins Wasser oder ans Land und wenn Land dann bleibt er da als trümmer und mit einem mini krater und wenn der auf ein anderes schiff landet dann verursacht er schaden. Aus jedem Schiff fliegen noch andere Gegenstände aber Turm verursacht am meisten schaden. Da muss ich mir überlegen was so auf dem Schiff noch als Fetzen fliegen kann. Wenn die Trümmer ins Wasser landen dann versenken die einfach.
Ich bin auch bisschen unter dem Eindruck von Spielen wie Bodycam, wo die Immersion eine grosse Rolle spielt. Mein Spiel soll zwar nicht realistisch sein, GAR nicht, aber ich will Elemente haben, die die Interaktion spannend, vielseitig und bisschen lustig machen.
Meine Ideen sind: wenn ein Marineflak ein leiches Boot zerstört explodiert er nicht sondern brennt und versenkt.
Wenn aber ein 8,8 cm Küstengeschütz leichtes Boot zerstört explodiert er und Turret fliegt hoch und siehe oben was passieren kann.
Wenn aber ein 15 cm leichtes Boot zerstört explodiert er wie verrückt und zerfällt in zwei, mehr Sachen fliegen um die Gegend und können theoretisch andere Boote beschädigen, vllt auch die Explosion an sich kann nahliegende Boote beschädigen!
Oder wenn ein leichtes Boot torpediert wird explodiert er und wenn bevor er komplett versinkt noch ein Torpedo oder ein Geschoss es trifft - explodiert er wie verrückt!
Standardmässig UI ist minimal, aber wenn man Tab klickt - toggle zu dem vollständigen UI mit Kanonen-Reichweiten, Ladebalken von Turrets, Torpedoanzahl des U-Boots, HP, Bewegungslinien, etc etc.

### MVP-Sprenger:
Die Kanonen-Schüsse sollen unterschiedlich und random sein, angelehnt an neues Bodycam-Spiel wo für ein Nachladen viele Animationen gibt was das Spiel bisschen lebendiger und insgesamt unterhaltsamer macht. Aus solchen Kleinigkeiten, erst dann ist das Spiel spannend meiner Meinung nach!
Screen-Shake wenn Schiffe explodieren und besonders wenn Torpedos treffen.
Unter dem Wasser bleibt das Siluett des versunkenen Schiffs.
Explodierte Schiffe verlieren Teile, die kann man später aufsammeln und dafür Perks oder was besonderes kaufen, Boosts oder interessante Upgrades. Das soll dazu motivieren, Schiffe übermässig zu zerstören, z.B. mit dem Uboot.
Möglichkeit, einem Turm ein bestimmtes Ziel zu geben. z.B. damit 15 cm Kanone auf kleinere Schiffe schiesst damit die explodieren. Vielleicht wenn mit einem Turm auf eine bestimmte Schiffart zielen wird er diese Typen immer als erste ins Visier nehmen und wenn zum UI Tab klicken steht dann neben dem Turm, dass er gerade eine bestimmte Schiffart priorisiert. Oder eventuell, falls einfacher, Modis implementieren... z.B. dass Kanonen entweder die ersten Schiffe priorisieren, des niedrigsten Ranges oder des höchsten Ranges. So kann man dann strategisch entscheiden.
Das Spiel soll in einem handgezeichneten Stil entwickelt werden. Stell dir das so vor: alle Assets sind auf einem Stück Papier von Hand gezeichnet mit einem schwarten Bleistift und Liner und mit einem roten Stift - Farbenpalette ist Vintage Propaganda.
Wenn du ein Assett siehst sieht es so aus als ob das von Hang gezeichnet wurde, mit einer Schere ausgeschnitten wurde und auf das Papier draufgelegt (sehr kleiner Schaten am Rand des Asset-Papierstücks). Wenn z.B. ein Enemy-Turm bei der Schiff-Detonation ans Land fliegt und da landet - wird auf der Landestelle mit einem Bleistift einen Krater dazugemahlt - diesmal nicht als ob der Krater auch ein Stück papier ist sondern als ob der Krater direkt auf dem Hauptleihnwand gezeichnet wurde.

* Fixed-size 2D map with abstract river route
* [Sharp river bends / tower-defense-style chokepoints
* [ ] Minimal permanent HUD
* [ ] Contextual UI only when interacting
* [ ] Left-click = world action / U-boat movement
* [ ] Right-click hold = radial context menu
* [ ] Space = fire torpedoes
* [ ] Enter = start next wave
* [ ] Tab hold = tactical information overlay
* [ ] Show turret build spots only contextually
* [ ] 3 turret types
* [ ] Turret placement via radial ghost blueprints
* [ ] Show turret range during placement
* [ ] Sell turret through contextual radial menu
* [ ] Radial menu structure for upgrades
* [ ] U-boat moves inside river in einer vordefinierten Linie in der Mitte. Wenn man mit linksklick nicht in der mitte klickt kommt Uboot in eine interpolierte stelle in der Mitte.
* [ ] U-boat can surface / dive
* [ ] U-boat can return to bunker
* [ ] Bunker repairs U-boat
* [ ] Bunker reloads torpedoes
* [ ] U-boat earns XP from sunk ships
* [ ] U-boat upgrades available after level-up
* [ ] Do not interrupt combat with upgrade screens
* [ ] U-Boot hat eine 8,8 Kanone oder kleine
* [ ] Destroyed U-boat can be rebuilt at bunker
* [ ] Unlimited preparation time between waves
* [ ] Separate combat and preparation states
* [ ] Show next-wave information during preparation
* [ ] Reichsmark displayed as paper/typewriter UI
* [ ] Show transactions briefly instead of permanent resource panels
* [ ] Keep XP visually associated with U-boat
* [ ] Show remaining torpedoes contextually
* [ ] Provide immediate feedback for invalid/empty actions
* [ ] Use cursor changes for interaction hints
* [ ] Avoid confirmation dialogs where hold/release interaction is enough
* [ ] Keep enemy route mostly hidden during combat
* [ ] Reveal tactical route/ranges with Tab
* [ ] Keep map readable without camera movement if possible
* [ ] Use vintage-propaganda-inspired visual language consistently
* [ ] Prioritize strong silhouettes and readability over realism
* [ ] Build input around gameplay intents, not individual entities
* [ ] Define input priority: modal UI > radial UI > world interaction
* [ ] Prepare architecture for future tower upgrades, perks and shop systems

## U-Boot
Die Zahlen muss man überholen, ist von ChatGPT generiert als Referenz
| Stat                       | I — VIIA |     II — VIIB |   III — VIIC |  IV-A — IXC | V-A — IXC Elite |    IV-B — VIIC/41 |        V-B — XXI |
| -------------------------- | -------: | ------------: | -----------: | ----------: | --------------: | ----------------: | ---------------: |
| **Rolle**                  |  Starter |    Allrounder |  Allrounder+ |    Gun/Tank |       Gun/Tank+ |            Hunter |        Submarine |
| **Upgrade-Kosten**         |    Start |        350 RM |       650 RM |    1.100 RM |        1.700 RM |          1.000 RM |         2.000 RM |
| **Benötigtes Level**       |        1 |             2 |            3 |           4 |               5 |                 4 |                5 |
| **HP**                     |      100 |           120 |          145 |         220 |             275 |               165 |          **105** |
| **Speed Oberfläche**       |     100% |          105% |         103% |         85% |             90% |              105% |         **115%** |
| **Speed getaucht**         |      72% |           74% |          72% |     **60%** |             65% |               82% |         **115%** |
| **Tauchzeit**              |    2.5 s |         2.3 s |        2.2 s |       3.0 s |           2.7 s |             1.7 s |        **0.9 s** |
| **Torpedos**               |        4 |             5 |            6 |           8 |              10 |                 7 |           **10** |
| **historische Rohre**      |      4+1 |           4+1 |          4+1 |         4+2 |             4+2 |               4+1 |        **6 Bug** |
| **Game-Salve**             |        1 |             1 |            1 |           1 |              2* |                 1 |            **2** |
| **Torpedo-DMG**            |      100 |           110 |          120 |         130 |             145 |               135 |          **150** |
| **Torpedo-Reload**         |    5.0 s |         4.8 s |        4.5 s |       5.0 s |           4.5 s |             4.0 s |        **3.0 s** |
| **Torpedo-Speed**          |     100% |          102% |         105% |        105% |            110% |              112% |         **125%** |
| **Deckkanone**             |   8,8 cm |        8,8 cm |       8,8 cm | **10,5 cm** |     **10,5 cm** |            8,8 cm |        **keine** |
| **Gun-DMG**                |       22 |            24 |           27 |      **42** |          **52** |                30 |                – |
| **Gun-Cooldown**           |    2.4 s |         2.2 s |        2.0 s |       2.8 s |           2.3 s |             1.8 s |                – |
| **Gun-Range**              |     100% |          105% |         110% |    **130%** |        **140%** |              115% |                – |
| **Gun während Tauchen**    |     Nein |          Nein |         Nein |        Nein |            Nein |              Nein |                – |
| **Schaden unter Wasser**   |     −60% |          −60% |         −65% |        −50% |            −55% |              −80% |    **0 / immun** |
| **Ram-/Collision-DMG**     |   normal |        normal |       normal |        −20% |            −25% |              −50% |   **0 getaucht** |
| **Repair-Speed im Bunker** |     100% |          100% |         100% |         80% |             75% |              110% |             125% |
| **Besonderheit**           |        – | mehr Munition | solider Kern |  Heavy Boat |     Doppelsalve | schnelles Tauchen | Unterwasserjäger |

## Brainstorming
Erstman gar ohne Geld anfangen oder nur für die kleinste Kanone, das es geradeso reicht oder dass man erst mit dem Uboot ein Boot versenken soll damit man genug geld hat?
Beim Abschiessen
