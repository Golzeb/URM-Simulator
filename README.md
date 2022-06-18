## URM Instructions

| Instruction  | Result |
| ---------- | ------ |
| Z(m)       | Put 0 in register 'm' |
| S(m)       | Increment register 'm' by 1 |
| T(m, n)    | Copy from register 'm' to register 'n' |
| I(m, n, o) | If register 'm' equals register 'n' jump to label 'o'[^label] | 

[^label]: Labels in this simulator are the equivalent to instruction numbers.


## Example program

Every URM program should start with the number of output register or '_' to indicate that all registers should be printed.

Following example adds two numbers from register 0 and register 1 and stores the output in register 4. 
```
4
0 : I(0, 2, 5)
1 : S(2)
2 : S(4)
3 : I(0, 2, 5)
4 : I(0, 0, 0)
5 : I(1, 3, stop)
6 : S(3)
7 : S(4)
8 : I(1, 3, stop)
9 : I(0, 0, 6)
stop: T(0, 0)
```

Simulator takes at least one argument which is the path to the urm program.
So if the program above was named `add.urm` you would run it with: `urm-simulator add.urm 4 5`.