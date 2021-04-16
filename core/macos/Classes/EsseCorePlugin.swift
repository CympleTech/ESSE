import Cocoa
import FlutterMacOS

public class EsseCorePlugin: NSObject, FlutterPlugin {
  public static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(name: "esse_core", binaryMessenger: registrar.messenger)
    let instance = EsseCorePlugin()
    registrar.addMethodCallDelegate(instance, channel: channel)
  }

  public func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    switch call.method {
    case "getPlatformVersion":
      result("macOS " + ProcessInfo.processInfo.operatingSystemVersionString)
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
