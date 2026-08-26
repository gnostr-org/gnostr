/// A container for typed application-defined app storage values
public struct AppStorageValues {
    private let provider: AppStorageProvider?

    /// Only to be used by AppStorage
    internal init(provider: AppStorageProvider?) {
        self.provider = provider
    }

    public func getValue<T: Codable & Sendable>(_ key: any AppStorageKey<T>.Type) -> T {
        guard let provider else { return key.defaultValue }
        return provider.getValue(key: key.name, defaultValue: key.defaultValue)
    }

    public func setValue<T: Codable & Sendable>(_ key: any AppStorageKey<T>.Type, newValue: T) {
        provider?.setValue(key: key.name, newValue: newValue)
    }
}
