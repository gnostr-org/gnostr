import LibP2P
import NIOConcurrencyHelpers

extension Application {
    public var autonat: AutoNATServices {
        .init(application: self)
    }

    public struct AutoNATServices: Sendable {
        public struct Provider {
            let run: @Sendable (Application) -> Void

            @preconcurrency public init(_ run: @Sendable @escaping (Application) -> Void) {
                self.run = run
            }
        }

        final class Storage: Sendable {
            let coordinator: NIOLockedValueBox<AutoNATCoordinator?>
            init() {
                self.coordinator = .init(nil)
            }
        }

        struct Key: StorageKey, Sendable {
            typealias Value = Storage
        }

        func initialize() {
            self.application.storage[Key.self] = .init()
        }

        public let application: Application

        var storage: Storage {
            if let storage = self.application.storage[Key.self] {
                return storage
            }
            let storage = Storage()
            self.application.storage[Key.self] = storage
            return storage
        }

        public var status: AutoNATStatus {
            self.storage.coordinator.withLockedValue { $0?.status ?? .unknown }
        }

        public func use(_ provider: Provider) {
            provider.run(self.application)
        }

        @preconcurrency public func use(_ makeCoordinator: @Sendable @escaping (Application) -> AutoNATCoordinator) {
            if self.application.storage[Key.self] == nil {
                self.initialize()
            }
            self.storage.coordinator.withLockedValue { $0 = makeCoordinator(self.application) }
            self.storage.coordinator.withLockedValue { $0?.install() }
        }
    }
}

extension Application.AutoNATServices.Provider {
    public static var autonat: Self {
        .init { app in
            app.autonat.use { AutoNATCoordinator(application: $0) }
        }
    }
}
