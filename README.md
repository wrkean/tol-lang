## Full Program Example
```plaintext
paraan factorial(n: numero) -> numero:
    kung n <= 1:
        ibalik 1
    ibalik n * factorial(n - 1)

paraan main():
    ang iiba bilang = 0
    ang limit: numero = 5

    habang bilang < limit:
        kung bilang == 3:
            print "Nilaktawan ang tatlo!"
            bilang = bilang + 1
            ituloy

        ang resulta = factorial(bilang)
        print "Ang factorial ay:"
        print resulta

        bilang = bilang + 1

    print "Tapos na!"

main();
```
