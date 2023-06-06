/*
Styr en 74HC595 shift register med en ATtiny13a (programmerad med Arduino Uno).

Resurser:
- 74HC595 guide: https://www.arduino.cc/en/Tutorial/Foundations/ShiftOut
- ATtiny13a med Arduino guide: https://create.arduino.cc/projecthub/taunoerik/programming-attiny13-with-arduino-uno-07beba
*/

//Pin connected to ST_CP of 74HC595
int latchPin = 4;
//Pin connected to SH_CP of 74HC595
int clockPin = 2;
////Pin connected to DS of 74HC595
int dataPin = 3;

byte outputs[5];

void setup() {
  //set pins to output so you can control the shift register
  pinMode(latchPin, OUTPUT);
  pinMode(clockPin, OUTPUT);
  pinMode(dataPin, OUTPUT);

  outputs[0] = 0x03; // 00000011 (red)
  outputs[1] = 0x0C; // 00001100 (yellow)
  outputs[2] = 0x30; // 00110000 (green)
  outputs[3] = 0x40; // 01000000 (white)
  outputs[4] = 0xFF; // 11111111 (all)

  display(outputs[4]);
}

void loop() {
  for (int i = 0; i < 6; i++) {
    display(outputs[i]);
  }
}

void display(byte n) {
  writeNum(n);
  delay(500);
}

void writeNum(byte n) {
  digitalWrite(latchPin, LOW);

  // shift out the bits:
  shiftOut(dataPin, clockPin, MSBFIRST, n);

  //take the latch pin high so the LEDs will light up:
  digitalWrite(latchPin, HIGH);
}
