import Flutter
import UIKit

public class SwiftEsseCorePlugin: NSObject, FlutterPlugin {
  public static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(name: "esse_core", binaryMessenger: registrar.messenger())
    let instance = SwiftEsseCorePlugin()
    registrar.addMethodCallDelegate(instance, channel: channel)
  }

  public func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    let url = URL(string: "http://www.apple.com")!
    let task = URLSession.shared.dataTask(with: url) { data, response, error in

    }
    task.resume()

    switch call.method {
    case "getPlatformVersion":
      result("iOS " + UIDevice.current.systemVersion)
    case "daemon":
      guard let args = call.arguments else {
        return
      }
      if let myArgs = args as? [String: Any],
         let path = myArgs["path"] as? String
      {
        DispatchQueue.global().async {
          start(path)
        }
        result("Daemon success")
      } else {
        result("Daemon path invalid")
      }
    default:
      result(FlutterMethodNotImplemented)
    }
  }
}
