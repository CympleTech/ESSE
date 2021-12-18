import 'package:flutter/material.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/widgets/socket_input.dart';
import 'package:esse/global.dart';
import 'package:esse/rpc.dart';

class NetworkDetail extends StatefulWidget {
  NetworkDetail({Key? key}) : super(key: key);

  @override
  _NetworkDetailState createState() => _NetworkDetailState();
}

class _NetworkDetailState extends State<NetworkDetail> {
  TextEditingController addController = TextEditingController();
  TextEditingController wsController = TextEditingController();
  TextEditingController httpController = TextEditingController();

  List<String> networkDht = [];
  List<List<String>> networkStable = [];

  String _selectedTrans = "quic";

  changeWs() async {
    Global.changeWs(wsController.text);
    await rpc.init(wsController.text);
    rpc.send('system-info', []);
    setState(() {});
  }

  changeHttp() {
    Global.changeHttp(httpController.text);
    setState(() {});
  }

  addBootstrap() {
    rpc.send('add-bootstrap', [addController.text, _selectedTrans]);
    setState(() {});
  }

  void loadNetworkDht() async {
    final res = await httpPost('network-dht', []);
    if (res.isOk) {
      this.networkDht.clear();
      res.params.forEach((p) {
        this.networkDht.add(p);
      });
      setState(() {});
    } else {
      // TODO tostor error
      print(res.error);
    }
  }

  void loadNetworkStable() async {
    final res = await httpPost('network-stable', []);
    if (res.isOk) {
      this.networkStable.clear();
      res.params.forEach((p) {
        this.networkStable.add([p[0], p[1]]);
      });
      setState(() {});
    } else {
      // TODO tostor error
      print(res.error);
    }
  }

  Widget _trans(String value, color) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Radio(
          value: value,
          groupValue: _selectedTrans,
          onChanged: (String? n) => setState(() {
              if (n != null) {
                _selectedTrans = n;
              }
        })),
        _selectedTrans == value
        ? Text(value.toUpperCase(), style: TextStyle(color: color.primary))
        : Text(value.toUpperCase()),
      ]
    );
  }

  @override
  initState() {
    loadNetworkStable();
    loadNetworkDht();
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    wsController.text = Global.wsRpc;
    httpController.text = Global.httpRpc;

    return Scaffold(
      appBar: AppBar(title: Text(lang.network)),
      body: SingleChildScrollView(
        child: Container(
          padding: const EdgeInsets.all(20.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              Row(),
              Container(
                width: 600.0,
                padding: const EdgeInsets.all(20.0),
                decoration: BoxDecoration(
                  color: Color(0xFF6174FF), borderRadius: BorderRadius.circular(15.0)),
                child: Row(children: [
                    Container(
                      width: 40.0,
                      height: 40.0,
                      margin: const EdgeInsets.only(right: 15.0),
                      decoration: BoxDecoration(
                        color: color.background,
                        borderRadius: BorderRadius.circular(15.0)),
                      child: Icon(Icons.info, color: Color(0xFF6174FF), size: 24.0),
                    ),
                    Expanded(
                      child: Text(
                        lang.deviceTip,
                        style: TextStyle(color: color.background, fontSize: 14.0),
                        textAlign: TextAlign.center,
                      ),
                    ),
              ])),
              const SizedBox(height: 20.0),
              Container(
                width: 600.0,
                padding: const EdgeInsets.symmetric(horizontal: 10.0),
                decoration: BoxDecoration(
                  color: const Color(0x26FF0000),
                  borderRadius: BorderRadius.circular(15.0)
                ),
                child: Column(
                  children: [
                    _settingHead(lang.deviceChangeWs),
                    SocketInputText(controller: wsController, action: changeWs, state: rpc.isLinked()),
                    _settingHead(lang.deviceChangeHttp),
                    SocketInputText(controller: httpController, action: changeHttp, state: true),
                    const SizedBox(height: 15.0),
                  ]
                )
              ),
              const SizedBox(height: 20.0),
              Container(
                width: 600.0,
                padding: const EdgeInsets.symmetric(horizontal: 10.0),
                decoration: BoxDecoration(
                  color: const Color(0x40ADB0BB),
                  borderRadius: BorderRadius.circular(15.0)
                ),
                child: Column(
                  children: [
                    _settingHead(lang.networkAdd),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                      children: [
                        _trans("quic", color), _trans("tcp", color),
                      ]
                    ),
                    const SizedBox(height: 5.0),
                    SocketInputText(controller: addController, action: addBootstrap, state: true),
                    const SizedBox(height: 15.0),
                  ]
                )
              ),
              _settingHead(lang.networkStable),
              Container(
                height: this.networkStable.length > 0 ? 100.0 : 50.0,
                width: 600.0,
                padding: const EdgeInsets.all(10.0),
                decoration: BoxDecoration(
                  color: const Color(0x40ADB0BB),
                  borderRadius: BorderRadius.circular(15.0)
                ),
                child: ListView.builder(
                  itemCount: this.networkStable.length,
                  itemBuilder: (context, index) {
                    final item = this.networkStable[index];
                    return Padding(
                      padding: const EdgeInsets.symmetric(vertical: 2.0),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(
                            item[1] == '1'
                            ? Icons.swap_horiz
                            : Icons.swap_calls,
                            size: 18.0,
                            color: color.primary),
                          SizedBox(width: 15.0),
                          Text(addrPrint(item[0]),
                            style: TextStyle(fontSize: 14.0))
                    ]));
                }),
              ),
              _settingHead(lang.networkDht),
              Container(
                height: this.networkDht.length > 0 ? 100.0 : 50.0,
                width: 600.0,
                padding: const EdgeInsets.all(10.0),
                decoration: BoxDecoration(
                  color: const Color(0x40ADB0BB),
                  borderRadius: BorderRadius.circular(15.0)
                ),
                child: ListView.builder(
                  itemCount: this.networkDht.length,
                  itemBuilder: (context, index) {
                    final item = this.networkDht[index];
                    return Padding(
                      padding: const EdgeInsets.symmetric(vertical: 2.0),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(Icons.cloud_done_outlined,
                            size: 18.0, color: color.primary),
                          SizedBox(width: 15.0),
                          Text(addrPrint(item),
                            style: TextStyle(fontSize: 14.0)),
                    ]));
                }),
              ),
          ])
    )));
  }
}

Widget _settingHead(String value) {
  return Padding(
      padding: const EdgeInsets.only(top: 10.0, bottom: 10.0),
      child: Text(value,
          style: TextStyle(fontSize: 16.0, fontWeight: FontWeight.bold)));
}
