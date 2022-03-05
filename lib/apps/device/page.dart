import 'dart:convert' show json;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:percent_indicator/percent_indicator.dart';
import 'package:qr_flutter/qr_flutter.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/input_text.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/show_pin.dart';
import 'package:esse/global.dart';
import 'package:esse/rpc.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/device/provider.dart';
import 'package:esse/apps/device/models.dart';

class DevicesPage extends StatefulWidget {
  @override
  _DevicesPageState createState() => _DevicesPageState();
}

class _DevicesPageState extends State<DevicesPage> {
  TextEditingController _addrController = TextEditingController();
  FocusNode _addrFocus = FocusNode();

  _inputAddress(lang) {
    showShadowDialog(context, Icons.devices_rounded, lang.addDevice,
      Column(
        children: [
          InputText(icon: Icons.location_on, text: "${lang.address} (0x...)",
            controller: this._addrController, focus: this._addrFocus),
          const SizedBox(height: 32.0),
          ButtonText(
            action: () {
              final addr = addrParse(this._addrController.text.trim());
              if (addr.length > 0) {
                Provider.of<DeviceProvider>(context, listen: false).connect(addr);
                Navigator.pop(context);
              }
            },
            text: lang.send,
          ),
        ]
    ));
  }

  _showQrCode(String name, String id, String lock, ColorScheme color, lang) async {
    final res = await httpPost('account-mnemonic', [lock]);
    if (res.isOk) {
      final words = res.params[0];
      final info = json.encode({'app': 'distribute', 'params': [name, pidText(id), words]});
      showShadowDialog(context, Icons.qr_code_rounded, lang.deviceQrcode,
        Column(
          children: [
            Container(
              width: 500.0,
              padding: const EdgeInsets.all(20.0),
              decoration: BoxDecoration(
                color: color.primary, borderRadius: BorderRadius.circular(15.0)),
              child: Row(children: [
                  Container(
                    width: 40.0,
                    height: 40.0,
                    margin: const EdgeInsets.only(right: 15.0),
                    decoration: BoxDecoration(
                      color: color.background,
                      borderRadius: BorderRadius.circular(15.0)),
                    child: Icon(Icons.info, color: color.primary, size: 24.0),
                  ),
                  Expanded(
                    child: Text(
                      lang.deviceQrcodeIntro,
                      style: TextStyle(color: color.background, fontSize: 14.0),
                      textAlign: TextAlign.center,
                    ),
                  ),
            ])),
            const SizedBox(height: 16.0),
            Container(
              width: 200.0,
              padding: const EdgeInsets.all(2.0),
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(5.0),
                border: Border.all(color: Color(0x40ADB0BB)),
                color: Colors.white,
              ),
              child: Center(
                child: QrImage(
                  data: info,
                  version: QrVersions.auto,
                  foregroundColor: Colors.black,
                ),
              ),
            ),
          ]
      ));
    } else {
      // TODO tostor error
      print(res.error);
    }
  }

  _scanQrCode() {
    // TODO
  }

  Widget deviceWidget(ColorScheme color, Device device, bool isDesktop, double widgetWidth, lang) {
    final bool isLocal = true; // TODO
    final String name = isLocal ? (device.name + " (${lang.deviceLocal})") : device.name;

    return Container(
      width: widgetWidth,
      decoration: BoxDecoration(
        color: (isLocal || device.online) ? color.primaryVariant : color.surface,
        borderRadius: BorderRadius.circular(15.0)),
      padding: const EdgeInsets.all(10.0),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          ListTile(
            leading: (isLocal || device.online)
            ? Icon(Icons.cloud_done_rounded, size: 38.0, color: Color(0xFF6174FF))
            : Icon(Icons.cloud_off_rounded, size: 38.0, color: Colors.grey),
            title: Text(name),
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: <Widget>[
              if (!isLocal && !device.online)
              TextButton(
                child: Text(lang.reconnect),
                onPressed: () {
                  Provider.of<DeviceProvider>(context, listen: false).connect(device.addr);
                },
              ),
              if (isLocal || device.online)
              TextButton(
                child: Text(lang.status),
                onPressed: () {
                  Provider.of<DeviceProvider>(context, listen: false).updateActivedDevice(device.id);
                  final widget = DeviceListenPage();
                  if (isDesktop) {
                    Provider.of<AccountProvider>(context, listen: false).updateActivedWidget(widget);
                  } else {
                    Navigator.push(context, MaterialPageRoute(builder: (_) => widget));
                  }
                },
              ),
              if (!isLocal)
              TextButton(
                child: Text(lang.delete, style: TextStyle(color: Colors.red)),
                onPressed: () {},
              ),
            ],
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);
    final provider = context.watch<DeviceProvider>();

    double widgetWidth = 300.0;
    if (isDesktop) {
      widgetWidth = (MediaQuery.of(context).size.width - 450) / 2;
    } else {
      widgetWidth = MediaQuery.of(context).size.width - 40;
    }

    final List<Widget> devicesWidgets = provider.devices.values.map((device) {
        return deviceWidget(color, device, isDesktop, widgetWidth, lang);
    }).toList();

    return Scaffold(
      appBar: AppBar(
        title: Text(lang.devices),
        actions: [
          PopupMenuButton<int>(
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(15)
            ),
            color: const Color(0xFFEDEDED),
            child: Icon(Icons.add_rounded, color: color.primary),
            onSelected: (int value) {
              if (value == 0) {
                // input address to connect.
                _inputAddress(lang);
              } else if (value == 1) {
                // show qrcode.
                final account = Provider.of<AccountProvider>(
                  context, listen: false).activedAccount;
                showShadowDialog(
                  context,
                  Icons.security_rounded,
                  lang.verifyPin,
                  PinWords(
                    pid: account.pid,
                    callback: (key) async {
                      Navigator.of(context).pop();
                      _showQrCode(
                        account.name,
                        account.pid,
                        key,
                        color,
                        lang,
                      );
                  }),
                  0.0
                );
              }
            },
            itemBuilder: (context) {
              return <PopupMenuEntry<int>>[
                PopupMenuItem<int>(value: 0,
                  child: Row(
                    children: [
                      Container(
                        padding: const EdgeInsets.only(right: 10.0),
                        width: 30.0,
                        height: 30.0,
                        child: Icon(Icons.add_rounded, color: Color(0xFF6174FF)),
                      ),
                      Text(lang.addDevice, style: TextStyle(color: Colors.black)),
                    ]
                  )
                ),
                PopupMenuItem<int>(value: 1,
                  child: Row(
                    children: [
                      Container(
                        padding: const EdgeInsets.only(right: 10.0),
                        width: 30.0,
                        height: 30.0,
                        child: Icon(Icons.qr_code_rounded, color: Color(0xFF6174FF)),
                      ),
                      Text(lang.deviceQrcode, style: TextStyle(color: Colors.black)),
                    ]
                  )
                ),
              ];
            }
          ),
          const SizedBox(width: 20.0),
        ]
      ),
      body: Padding(
        padding: const EdgeInsets.all(20.0),
        child: SingleChildScrollView(
          child: Wrap(
            spacing: 16.0,
            runSpacing: 16.0,
            alignment: WrapAlignment.start,
            children: devicesWidgets
          )
        )
      )
    );
  }
}

class DeviceListenPage extends StatefulWidget {
  @override
  _DeviceListenPageState createState() => _DeviceListenPageState();
}

class _DeviceListenPageState extends State<DeviceListenPage> {
  Widget percentWidget(double cpu_p, String cpu_u, double radius, Color color) {
    return Container(
      margin: const EdgeInsets.symmetric(vertical: 10.0),
      width: radius * 2,
      alignment: Alignment.center,
      child: CircularPercentIndicator(
        radius: radius,
        lineWidth: 16.0,
        animation: true,
        percent: cpu_p/100,
        center: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Text("${cpu_p}%", style: TextStyle(fontWeight: FontWeight.bold, fontSize: 20.0)),
            const SizedBox(height: 4.0),
            Text(cpu_u)
          ]
        ),
        circularStrokeCap: CircularStrokeCap.round,
        progressColor: color,
      )
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);
    final status = context.watch<DeviceProvider>().status;
    final uptimes = status.uptime.uptime();

    double radius = MediaQuery.of(context).size.width / 4 - 10;
    if (radius > 100) {
      radius = 100;
    } else {
      radius = MediaQuery.of(context).size.width / 2 - 50;
      if (radius > 99) {
        radius = 99;
      }
    }

    final w1 = percentWidget(
      status.cpu_p(), "CPU: ${status.cpu_u()} cores", radius, Color(0xFF6174FF),
    );
    final w2 = percentWidget(
      status.memory_p(), "${lang.memory}: ${status.memory_u()}", radius, Colors.blue,
    );
    final w3 = percentWidget(
      status.swap_p(), "${lang.swap}: ${status.memory_u()}", radius, Colors.green,
    );
    final w4 = percentWidget(
      status.disk_p(), "${lang.disk}: ${status.disk_u()}", radius, Colors.purple,
    );

    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(10.0),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.start,
            crossAxisAlignment: isDesktop ? CrossAxisAlignment.start : CrossAxisAlignment.center,
            children: <Widget>[
              Row(
                mainAxisAlignment: MainAxisAlignment.start,
                children: [
                  GestureDetector(
                    onTap: () {
                      Provider.of<DeviceProvider>(context, listen: false).clear();
                      if (isDesktop) {
                        Provider.of<AccountProvider>(context, listen: false).updateActivedWidget(DevicesPage());
                      } else {
                        Navigator.pop(context);
                      }
                    },
                    child: Container(width: 20.0,
                      child: Icon(Icons.arrow_back, color: color.primary)),
                  ),
                  Expanded(
                    child: Text(
                      "${lang.uptime}: ${uptimes[0]} ${lang.days}, ${uptimes[1]} ${lang.hours}, ${uptimes[2]} ${lang.minutes}",
                      style: TextStyle(fontWeight: FontWeight.bold),
                      textAlign: TextAlign.right,
                    ),
                  ),
                  const SizedBox(height: 10.0),
                ]
              ),
              const SizedBox(height: 20.0),
              Expanded(
                child: SingleChildScrollView(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children:
                    radius == 100 ? [
                      Row(mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                        children: [w1, w2]
                      ),
                      const SizedBox(height: 40.0),
                      Row(mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                        children: [w3, w4]
                      ),
                    ] : [w1, w2, w3, w4]
                  ),
                )
              )
            ]
          )
        )
      )
    );
  }
}
