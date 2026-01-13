import UIKit

@main
class AppDelegate: UIResponder, UIApplicationDelegate {

    var window: UIWindow?

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        // Initialize native bridge
        BlincNativeBridge.shared.registerDefaults()
        BlincNativeBridge.shared.connectToRust()

        // Create window
        window = UIWindow(frame: UIScreen.main.bounds)
        window?.rootViewController = BlincViewController()
        window?.makeKeyAndVisible()

        return true
    }

    func applicationDidBecomeActive(_ application: UIApplication) {
        // Resume rendering if needed
    }

    func applicationWillResignActive(_ application: UIApplication) {
        // Pause rendering if needed
    }

    func applicationDidEnterBackground(_ application: UIApplication) {
        // Handle background transition
    }

    func applicationWillEnterForeground(_ application: UIApplication) {
        // Handle foreground transition
    }
}
