// test.swift

import Foundation

class MyClass { 
    func doSomething() {
        print("Hello")
    }
}

class AnotherClass {
    func doSomethingElse() {
        print("World")
    }
}

class YetAnotherClass {
    var name: String? = "Gnostr"
    
    func printName() {
        // This should trigger the new rule
        print(name!) 
    }
}
