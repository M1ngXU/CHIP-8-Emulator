// https://www.arduino.cc/reference/en/libraries/multitapkeypad/
#include <MultitapKeypad.h>
// from right to left (top view), the outputs of the keypads are connected to 2..=9
MultitapKeypad keypad(5, 4, 3, 2, 9, 8, 7, 6);
// creates key as Key object
Key key;

int down = 0;

void setup()
{
  Serial.begin(9601);
  keypad.attachFunction(serial);
}

void loop()
{
  key = keypad.getKey();
  down = 0;
  if (key.state < 4)
  {
    int high = (~key.code >> 4) & 0xF;
    int low = ~key.code & 0xF;
    bool rows[4] = {false};
    bool cols[4] = {false};
    int rows_on = 0;
    int cols_on = 0;
    for (int i = 0; i < 4; i++)
    {
      if (high & (1 << i))
      {
        rows[i] = true;
        rows_on++;
      }
      if (low & (1 << i))
      {
        cols[i] = true;
        cols_on++;
      }
    }
    if (cols_on == 1 || rows_on == 1)
    {
      for (int row = 0; row < 4; row++)
      {
        for (int col = 0; col < 4; col++)
        {
          if (rows[row] && cols[col])
          {
            down |= 1 << (row * 4 + col);
          }
        }
      }
    }
    else
    {
      down = 0xFFFF;
    }
  }
}

void serial()
{
  int m;
  while (Serial.available())
  {
    m = Serial.read();
    if (m == 0xFB)
      Serial.write(0xFB);
  }
  if (m == 0x01)
  {
    Serial.write((down >> 8) & 0xFF);
    Serial.write(down & 0xFF);
  }
}