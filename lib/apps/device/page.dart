import 'dart:convert' show json;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:percent_indicator/percent_indicator.dart';
import 'package:qr_flutter/qr_flutter.dart';

import 'package:esse/utils/adaptive.dart';
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
              var addr = this._addrController.text;
              if (addr.substring(0, 2) == '0x') {
                //substring(2); if has 0x, need remove
                addr = addr.substring(2);
              }
              if (addr.length > 0) {
                Provider.of<DeviceProvider>(context, listen: false).connect(addr);
                Navigator.pop(context);
              }
            },
            text: lang.send,
            width: 600.0
          ),
        ]
    ));
  }

  _showQrCode(String name, String id, String addr, String lock, ColorScheme color, lang) async {
    final res = await httpPost(Global.httpRpc, 'account-mnemonic', [lock]);
    if (res.isOk) {
      final words = res.params[0];
      final info = json.encode({'app': 'distribute', 'params': [name, id, addr, words]});
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
    final bool isLocal = '0x' + device.addr == Global.addr;
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
            subtitle: Container(
              padding: const EdgeInsets.only(top: 8.0),
              child: Text(device.printAddr())
            ),
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
    }

    final List<Widget> devicesWidgets = provider.devices.values.map((device) {
        return deviceWidget(color, device, isDesktop, widgetWidth, lang);
    }).toList();

    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(10.0),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.start,
            crossAxisAlignment: isDesktop ? CrossAxisAlignment.start : CrossAxisAlignment.center,
            children: <Widget>[
              Row(
                children: [
                  if (!isDesktop)
                  GestureDetector(
                    onTap: () => Navigator.pop(context),
                    child: Container(width: 20.0, child: Icon(Icons.arrow_back, color: color.primary)),
                  ),
                  const SizedBox(width: 15.0),
                  Expanded(child: Text(lang.devices, style: TextStyle(fontWeight: FontWeight.bold, fontSize: 20.0))),
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
                            hashPin: account.lock,
                            callback: (_key, _hash) async {
                              Navigator.of(context).pop();
                              _showQrCode(
                                account.name,
                                account.id,
                                Global.addr,
                                account.lock,
                                color,
                                lang,
                              );
                        }));
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

                  const SizedBox(width: 10.0),
                ],
              ),
              const SizedBox(height: 30.0),
              Expanded(
                child: SingleChildScrollView(
                  child: Wrap(
                    spacing: 16.0,
                    runSpacing: 16.0,
                    alignment: WrapAlignment.start,
                    children: devicesWidgets
                  )
                )
              ),
            ]
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
      width: radius + 10,
      alignment: Alignment.center,
      child: CircularPercentIndicator(
        radius: radius,
        lineWidth: 16.0,
        animation: true,
        percent: cpu_p/100,
        center: Text("${cpu_p}%",
          style: TextStyle(fontWeight: FontWeight.bold, fontSize: 20.0),
        ),
        footer: Padding(
          padding: const EdgeInsets.only(top: 8.0, bottom: 32.0),
          child: Text(cpu_u,
            style: TextStyle(fontWeight: FontWeight.bold, fontSize: 17.0),
          ),
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

    double radius = MediaQuery.of(context).size.width / 2 - 40;
    if (radius > 150) {
      radius = 150;
    }

    double height = MediaQuery.of(context).size.height / 2 - radius - 60;
    if (height < 16) {
      height = 16;
    } else if (!isDesktop) {
      height = 32;
    }

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
              SizedBox(height: height),
              Expanded(
                child: SingleChildScrollView(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                        children: [
                          percentWidget(
                            status.cpu_p(),
                            "CPU: ${status.cpu_u()} cores",
                            radius,
                            Color(0xFF6174FF),
                          ),
                          percentWidget(
                            status.memory_p(),
                            "${lang.memory}: ${status.memory_u()}",
                            radius,
                            Colors.blue,
                          ),
                        ]
                      ),
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                        children: [
                          percentWidget(
                            status.swap_p(),
                            "${lang.swap}: ${status.memory_u()}",
                            radius,
                            Colors.green,
                          ),
                          percentWidget(
                            status.disk_p(),
                            "${lang.disk}: ${status.disk_u()}",
                            radius,
                            Colors.purple,
                          ),
                        ]
                      ),
                    ]
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
