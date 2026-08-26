import LibP2P
import NIOConcurrencyHelpers

extension Application {
    public var relay: RelayServices {
        .init(application: self)
    }

    public struct RelayServices: Sendable {
        public struct Provider {
            let run: @Sendable (Application) -> Void

            @preconcurrency public init(_ run: @Sendable @escaping (Application) -> Void) {
                self.run = run
            }
        }

        final class Storage: Sendable {
            let coordinator: NIOLockedValueBox<RelayCoordinator?>
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

        public func use(_ provider: Provider) {
            provider.run(self.application)
        }

        @preconcurrency public func use(_ makeCoordinator: @Sendable @escaping (Application) -> RelayCoordinator) {
            if self.application.storage[Key.self] == nil {
                self.initialize()
            }
            self.storage.coordinator.withLockedValue { $0 = makeCoordinator(self.application) }
            self.storage.coordinator.withLockedValue { $0?.install() }
        }
    }
}

extension Application.RelayServices.Provider {
    public static var relay: Self {
        .init { app in
            app.relay.use { RelayCoordinator(application: $0) }
        }
    }
}
