@propertyWrapper
struct UncheckedSendable<T>: @unchecked Sendable {
    var wrappedValue: T
}
