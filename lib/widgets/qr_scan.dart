import 'dart:io';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter/material.dart';
import 'package:qr_code_scanner/qr_code_scanner.dart';

class QRScan extends StatefulWidget {
  final Function callback;
  const QRScan({Key? key, required this.callback}) : super(key: key);

  @override
  State<StatefulWidget> createState() => _QRScanState();
}

class _QRScanState extends State<QRScan> {
  QRViewController? controller;
  final GlobalKey qrKey = GlobalKey(debugLabel: 'QR');

  // In order to get hot reload to work we need to pause the camera if the platform
  // is android, or resume the camera if the platform is iOS.
  @override
  void reassemble() {
    super.reassemble();
    if (Platform.isAndroid) {
      controller?.pauseCamera();
    } else if (Platform.isIOS) {
      controller?.resumeCamera();
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: AnnotatedRegion<SystemUiOverlayStyle>(
        value: SystemUiOverlayStyle.light.copyWith(statusBarColor: Colors.black),
        child: SafeArea(
          child: Column(
            children: <Widget>[
              Container(
                height: 60.0,
                color: Colors.black,
                padding: const EdgeInsets.symmetric(horizontal: 30.0, vertical: 20.0),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
                    GestureDetector(
                      onTap: () => Navigator.pop(context),
                      child: Icon(Icons.arrow_back, color: Colors.white, size: 26.0)
                    ),
                    GestureDetector(
                      onTap: () => controller?.flipCamera(),
                      child: Icon(Icons.flip_camera_ios_outlined, color: Colors.white, size: 26.0)
                    ),
                  ]
                )
              ),
              Expanded(child: _buildQrView(context)),
              Container(
                height: 70.0,
                color: Colors.black,
                padding: const EdgeInsets.only(top: 20.0, bottom: 20.0),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
                    GestureDetector(
                      onTap: () async {
                        await controller?.toggleFlash();
                        setState(() {});
                      },
                      child: FutureBuilder(
                        future: controller?.getFlashStatus(),
                        builder: (context, snapshot) {
                          if (snapshot.data != null) {
                            return Icon(Icons.highlight_outlined, color: Colors.white, size: 30.0);
                          } else {
                            return Icon(Icons.lightbulb_outline_rounded, color: Colors.white, size: 30.0);
                          }
                        },
                      )
                    ),
                  ]
                )
              ),
            ],
          ),
    )));
  }

  Widget _buildQrView(BuildContext context) {
    // For this example we check how width or tall the device is and change the scanArea and overlay accordingly.
    var scanArea = (MediaQuery.of(context).size.width < 400 ||
            MediaQuery.of(context).size.height < 400)
        ? 200.0
        : 350.0;
    // To ensure the Scanner view is properly sizes after rotation
    // we need to listen for Flutter SizeChanged notification and update controller
    return QRView(
      key: qrKey,
      onQRViewCreated: _onQRViewCreated,
      overlay: QrScannerOverlayShape(
          borderColor: Colors.red,
          borderRadius: 10,
          borderLength: 30,
          borderWidth: 10,
          cutOutSize: scanArea),
    );
  }

  void _onQRViewCreated(QRViewController controller) async {
    this.controller = controller;
    controller.scannedDataStream.listen((scanData) async {
      if (scanData.code == null) {
        return;
      }
      final Map qrInfo = json.decode(scanData.code!);
      if (!qrInfo.containsKey("app") || !qrInfo.containsKey("params")) {
        // TODO show Error.
        return;
      }
      await this.controller?.pauseCamera();
      widget.callback(true, qrInfo["app"], qrInfo["params"]);
    });
  }

  @override
  void dispose() {
    this.controller?.dispose();
    super.dispose();
  }
}
