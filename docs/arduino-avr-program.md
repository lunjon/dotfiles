# AVR Programmering med Arduino

Du kan använda din arduino för att programmera dina mcu:er med AVR.

Följande är baserat på [denna](https://github.com/sleemanj/optiboot/blob/master/dists/README.md).

## ATtiny13A

Alternativt kan du använda [MicroCore](https://github.com/MCUdude/MicroCore) som är specifikt för ATtiny13.

### Koppling

Börja med följande kopplingar för att kunna programmera:
- Arduino 5V -> pin 8
- GND -> pin 4
- Arduino pin 13 -> pin 7
- Arduino pin 12 -> pin 6
- Arduino pin 11 -> pin 5
- Arduino pin 10 -> pin 1

### Arduino ISP

För att kunna använda arduino:n för programmering måste du ladda uppen sketch för det.
I arduino IDE:n välj följande: `Files -> Examples -> ArduinoISP`

Ladda sedan upp detta till arduino:n.

### Installera Core Files

Detta behövs för själva programmeringen. Gå till `Files -> Preferences`
och klistra in under "Additional boards manager URLs:": `https://raw.githubusercontent.com/sleemanj/optiboot/master/dists/package_gogo_diy_attiny_index.json`
Flera URL:er kan separeras med komma.

Gå sedan till `Tools -> Board -> Boards Manager ...` och sök efter "DIY ATtiny" (i kolumnen till vänster). Installera den.

Välj sedan `Tools -> Board -> DIY ATtiny -> ATtiny13`.

**OBS!** Se till att `Tools -> Programmer` är satt till "Arduino as ISP".

### Ladda upp sketch

Nu borde allt vara redo för att laddas upp. Välj (eller skriv) vilken sketch du vill ladda upp.
Använd sedan `Sketch -> Upload Using Progammer` för att ladda upp.

**OBS!** Glöm inte kopplingen ovan och att du, efter sketchen laddats upp, måste kopplas om.