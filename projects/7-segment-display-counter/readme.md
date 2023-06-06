# 7-segment display

Räknar en 7-segment display från 0-9 mha. av drivare av typen 74HC4511.

## Komponenter
- Arduino (testad på nano)
- 74HC4511
- Breadboard
- 12+ kablar
- 7 motstånd á 470 ohm

## Förbättringar och utökning
- Snygga till kopplingen
- Lägg till knapp där tryckning ökar siffrar
  - enkelklick ökar med 1
  - dubbelklick minskar med 1

## Detaljer
A, B, C och D motsvarar pins som finns på 74HC4511.
I ordningen DCBA så motsvarar de binära tal.

Exempel:

    D | C | B | A = N
    -----------------
    0   0   0   0 = 0
    0   0   1   0 = 2
    1   0   0   1 = 9
