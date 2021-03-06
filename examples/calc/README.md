# Calc
Пример простого калькулятора арифметических выражений.<br>
Распознает операции сложения (+), вычитания (-), умножения (\*), деления (/), возведения в степень (^) и унарный минус. Также распознает следующие функции: *abs*, *sin*, *cos*, *tan*, *exp*, *log2*. Для группировки можно использовать круглые скобки.

[putty]: https://www.chiark.greenend.org.uk/~sgtatham/putty/

## Запуск сервера
В папке с установленным *Message reader* выполнить следующую команду:

    node node_modules/message-reader/examples/calc

## Проверка работы
Проверку можно выполнить, например, используя [*PuTTY*][putty].
- Для этого запустить *PuTTY*. Подключиться к серверу, параметры подключения (Category -> Session):
  - Host Name (or IP address): **localhost**
  - Port: **5555**
  - Connection type: **Raw**
- В окне терминала *PuTTY* набрать какое-либо арифметическое выражение, например:

      2 * (3 + 4)

  и нажать *Enter*. После этого в окне терминала *PuTTY* должен отобразиться результат вычисления выражения:

      result: 14