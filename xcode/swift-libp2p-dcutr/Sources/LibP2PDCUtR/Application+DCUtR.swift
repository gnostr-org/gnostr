import LibP2P
import NIOConcurrencyHelpers

extension Application {
    public var dcutr: DCUtRServices {
        .init(application: self)
    }

    public struct DCUtRServices: Sendable {
        public struct Provider {
            let run: @Sendable (Application) -> Void

            @preconcurrency public init(_ run: @Sendable @escaping (Application) -> Void) {
                self.run = run
            }
        }

        final class Storage: Sendable {
            let coordinator: NIOLockedValueBox<DCUtRCoordinator?>
            init() {
                self.coordinator = .init(nil)
            }
        }

        struct Key: StorageKey {
            typealias Value = Storage
        }

        func initialize() {
            self.application.storage[Key.self] = .init()
        }

        public let application: Application

        var storage: Storage {
            guard let storage = self.application.storage[Key.self] else {
                fatalError("DCUtR not initialized. Configure with app.dcutr.initialize()")
            }
            return storage
        }

        public func use(_ provider: Provider) {
            provider.run(self.application)
        }

        @preconcurrency func use(_ makeCoordinator: @Sendable @escaping (Application) -> DCUtRCoordinator) {
            if self.application.storage[Key.self] == nil {
                self.initialize()
            }
            self.storage.coordinator.withLockedValue { $0 = makeCoordinator(self.application) }
            self.storage.coordinator.withLockedValue { $0?.install() }
        }
    }
}

extension Application.DCUtRServices.Provider {
    public static var dcutr: Self {
        .init { app in
            app.dcutr.use { DCUtRCoordinator(application: $0) }
        }
    }
}
