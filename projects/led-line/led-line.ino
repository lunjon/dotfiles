/*
LED Line
========
Styr en rad med 8 LEDs mha. ett 74HC595 shift register.

TODO
====
- variera delay med trimpot/potentiometer
- ändra program med en knapp

Komponenter
===========

Resurser
========
- 74HC595 guide: https://www.arduino.cc/en/Tutorial/Foundations/ShiftOut
*/

#define dataPin 4   // Pin connected to DS of 74HC595
#define latchPin 3  // Pin connected to ST_CP of 74HC595
#define clockPin 2  // Pin connected to SH_CP of 74HC595

unsigned long DELAY = 50;
int currentProgram = 2;

void setup() {
  // Set pins to output so you can control the shift register
  pinMode(dataPin, OUTPUT);
  pinMode(latchPin, OUTPUT);
  pinMode(clockPin, OUTPUT);
}

void loop() {
  uint8_t size;         // Size of the allocated array in bytes
  byte* output;         // Array containing the program
  bool mirror = false;  // Run a reverse of the program as well

  if (currentProgram == 0) {
    // Runs up and down
    size = 8;
    mirror = true;
    output = createArray(size);
    byte* p = output;
    *p++ = 0x01;
    *p++ = 0x02;
    *p++ = 0x04;
    *p++ = 0x08;
    *p++ = 0x10;
    *p++ = 0x20;
    *p++ = 0x40;
    *p++ = 0x80;
  } else if (currentProgram == 1) {
    // Just blinks all LEDs
    size = 4;
    output = createArray(size);
    byte* p = output;
    *p++ = 0x00;
    *p++ = 0x00;
    *p++ = 0xFF;
    *p++ = 0xFF;
  } else if (currentProgram == 2) {
    // Fill up and down
    size = 8;
    mirror = true;
    output = createArray(size);
    byte* p = output;
    *p++ = 0x00;
    *p++ = 0x01;
    *p++ = 0x03;
    *p++ = 0x0F;
    *p++ = 0x1F;
    *p++ = 0x3F;
    *p++ = 0x7F;
    *p++ = 0xFF;
  }

  if (output == NULL) {
    delay(10);
    return;
  }

  // Runs from first to last element
  byte* p = output;
  for (uint8_t i = 0; i < size; i++) {
    display(*p);
    *p++;
    delay(DELAY);
  }

  // If mirror = true, run from last - 1 to first element
  if (mirror) {
    *p--;
    for (uint8_t i = 0; i < size - 1; i++) {
      display(*p);
      *p--;
      delay(DELAY);
    }
  }

  // Free allocated memory
  if (output != NULL) {
    free(output);
  }
}

void display(byte n) {
  digitalWrite(latchPin, LOW);

  // shift out the bits:
  shiftOut(dataPin, clockPin, MSBFIRST, n);

  // Take the latch pin high so the LEDs will light up
  digitalWrite(latchPin, HIGH);
}

byte* createArray(uint8_t size) {
  byte* p = (byte*)malloc(size * sizeof(byte));
  return p;
}
