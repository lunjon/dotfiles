int A = 4;
int B = 5;
int C = 6;
int D = 7;

void setup() {
  // Configure pins
  pinMode(A, OUTPUT);
  pinMode(B, OUTPUT);
  pinMode(C, OUTPUT);
  pinMode(D, OUTPUT);
}

void loop() {
  for (byte i = 0; i < 10; i++) {
    setDisplay(i);
    delay(250);
  }
}

/* Sets driver input values for the given number.
Only works for 0-9. */
void setDisplay(byte n) {
  setPin(A, 0x01, n);
  setPin(B, 0x02, n);
  setPin(C, 0x04, n);
  setPin(D, 0x08, n);
}

// Set output of pin given a control byte and number n (0-9).
void setPin(int pin, byte c, byte n) {
  if (c & n) {
    digitalWrite(pin, HIGH);
  } else {
    digitalWrite(pin, LOW);
  }
}