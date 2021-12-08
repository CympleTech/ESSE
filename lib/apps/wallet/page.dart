import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/input_text.dart';
import 'package:esse/widgets/default_core_show.dart';
import 'package:esse/global.dart';
import 'package:esse/options.dart';
import 'package:esse/rpc.dart';

import 'package:esse/apps/wallet/models.dart';

class WalletDetail extends StatefulWidget {
  const WalletDetail({Key? key}) : super(key: key);

  @override
  _WalletDetailState createState() => _WalletDetailState();
}

class _WalletDetailState extends State<WalletDetail> with SingleTickerProviderStateMixin {
  TabController? _tabController;
  bool _needGenerate = false;

  List<Address> _addresses = [];
  Address? _selectedAddress;

  List<Network> _networks = [];
  Network? _selectedNetwork;

  List tokens = [
    ['ETH', '100', '1000', 'assets/logo/logo_eth.png'],
    ['USDT', '2000', '2000', 'assets/logo/logo_tether.png'],
    ['XXX', '100', '1000', 'assets/logo/logo_erc20.png'],
    ['FFF', '100', '1000', 'assets/logo/logo_erc20.png'],
  ];

  @override
  void initState() {
    _tabController = new TabController(length: 2, vsync: this);
    rpc.addListener('wallet-generate', _walletGenerate, false);
    rpc.addListener('wallet-balance', _walletBalance, false);
    super.initState();
    Future.delayed(Duration.zero, _load);
  }

  _walletGenerate(List params) {
    final address = Address.fromList(params);
    bool isNew = true;
    this._addresses.forEach((addr) {
        if (addr.address == address.address) {
          isNew = false;
        }
    });
    if (isNew) {
      this._addresses.add(address);
      setState(() {});
    }
  }

  _walletBalance(List params) {
    //
  }

  _load() async {
    final res = await httpPost(Global.httpRpc, 'wallet-list', []);
    if (res.isOk) {
      this._addresses.clear();
      res.params.forEach((param) {
          this._addresses.add(Address.fromList(param));
      });
      if (this._addresses.length == 0) {
        this._needGenerate = true;
      }
      setState(() {});
    } else {
      // TODO tostor error
      print(res.error);
    }
  }

  _changeAddress(Address address) {
    this._selectedAddress = address;
    this._networks = address.networks();
    if (!this._networks.contains(this._selectedNetwork)) {
      this._selectedNetwork = this._networks[0];
    }
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    if (this._addresses.length == 0 && !this._needGenerate) {
      return Scaffold(
        appBar: AppBar(title: Text(lang.loadMore)),
        body: const DefaultCoreShow(),
      );
    }

    if (this._addresses.length == 0 && this._needGenerate) {
      return Scaffold(
        appBar: AppBar(title: Text(lang.wallet)),
        body: DefaultCoreShow(
          child: ElevatedButton(
            style: ElevatedButton.styleFrom(onPrimary: color.surface),
            onPressed: () {
              rpc.send('wallet-generate', [ChainToken.ETH.toInt(), ""]);
            },
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 16.0),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Icon(Icons.lock),
                  const SizedBox(width: 8.0),
                  const Text('生成以太坊地址'),
                ]
              )
            )
        ))
      );
    }

    if (this._selectedAddress == null) {
      _changeAddress(this._addresses[0]);
    }

    return Scaffold(
      appBar: AppBar(
        title: DropdownButton<Network>(
          icon: Container(),
          underline: Container(),
          value: this._selectedNetwork,
          onChanged: (Network? value) {
            if (value != null) {
              setState(() {
                  this._selectedNetwork = value;
              });
            }
          },
          items: this._networks.map((Network network) {
              final params = network.params();
              return  DropdownMenuItem<Network>(
                value: network,
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 6.0),
                  decoration:  BoxDecoration(
                    border: Border.all(width: 1.0, color: params[1]),
                    borderRadius: BorderRadius.circular(25.0)
                  ),
                  child: Row(
                    children: <Widget>[
                      Icon(Icons.public, color: params[1], size: 18.0),
                      const SizedBox(width: 10),
                      Text(params[0], style: TextStyle(color: params[1], fontSize: 14.0)),
                    ],
                )),
              );
          }).toList(),
        ),
        actions: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 10.0),
            child: DropdownButton<Address>(
              icon: Container(),
              underline: Container(),
              value: this._selectedAddress,
              onChanged: (Address? value) {
                if (value != null) {
                  setState(() {
                      _changeAddress(value);
                  });
                }
              },
              items: this._addresses.map((Address addr) {
                  return  DropdownMenuItem<Address>(
                    value: addr,
                    child: Container(
                      margin: const EdgeInsets.symmetric(horizontal: 10.0),
                      width: 40.0,
                      height: 40.0,
                      decoration:  BoxDecoration(
                        color: color.surface,
                        borderRadius: BorderRadius.circular(25.0)
                      ),
                      child: Center(child: Text(addr.icon()))
                    ),
                  );
              }).toList(),
            ),
          )
        ]
      ),
      body: Container(
        alignment: Alignment.topCenter,
        padding: const EdgeInsets.symmetric(horizontal: 20.0),
        child: Column(
          children:[
            InkWell(
              onTap: () {
                Clipboard.setData(ClipboardData(text: this._selectedAddress!.address));
              },
              child: Container(
                padding: const EdgeInsets.symmetric(vertical: 10.0),
                alignment: Alignment.center,
                decoration: new BoxDecoration(
                  border: new Border(bottom:
                    const BorderSide(width: 1.0, color: Color(0xA0ADB0BB)))),
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Text(this._selectedAddress!.name, style: TextStyle(fontSize: 18.0)),
                    const SizedBox(height: 4.0),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Text(this._selectedAddress!.short(),
                          style: TextStyle(color: Color(0xFFADB0BB))),
                        const SizedBox(width: 8.0),
                        Icon(Icons.copy, size: 16.0, color: color.primary),
                      ]
                    )
                  ]
                ),
              ),
            ),
            Container(
              padding: const EdgeInsets.symmetric(vertical: 20.0),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Container(
                    width: 36.0,
                    height: 36.0,
                    decoration: BoxDecoration(
                      image: DecorationImage(
                        image: AssetImage(tokens[0][3]),
                        fit: BoxFit.cover,
                      ),
                    ),
                  ),
                  Container(
                    height: 60.0,
                    alignment: Alignment.center,
                    child: Text(
                      '100 ETH',
                      style: TextStyle(fontSize: 24.0, fontWeight: FontWeight.bold)),
                  ),
                  Text('\$1000', style: TextStyle(color: Color(0xFFADB0BB))),
                  const SizedBox(height: 8.0),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                    children: [
                      TextButton(
                        onPressed: () {
                          setState(() {});
                        },
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 10.0),
                          decoration:  BoxDecoration(
                            color: Color(0xFF6174FF),
                            borderRadius: BorderRadius.circular(25.0)
                          ),
                          child: Center(child: Text('Send', style: TextStyle(color: Colors.white)))
                        )
                      ),
                      TextButton(
                        onPressed: () {
                          setState(() {});
                        },
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 10.0),
                          decoration:  BoxDecoration(
                            color: Color(0xFF6174FF),
                            borderRadius: BorderRadius.circular(25.0)
                          ),
                          child: Center(child: Text('Receive', style: TextStyle(color: Colors.white)))
                        )
                      ),
                    ]
                  ),
                ]
              )
            ),
            TabBar(
              unselectedLabelColor: color.onSurface,
              labelColor: Color(0xFF6174FF),
              tabs: [
                Tab(text: 'Assets'),
                Tab(text: 'Activity'),
              ],
              controller: _tabController!,
              indicatorSize: TabBarIndicatorSize.tab,
            ),
            Expanded(
              child: TabBarView(
                children: [
                  ListView.separated(
                    separatorBuilder: (BuildContext context, int index) => const Divider(),
                    itemCount: tokens.length,
                    itemBuilder: (BuildContext context, int index) {
                      return ListTile(
                        leading: Container(
                          width: 36.0,
                          height: 36.0,
                          decoration: BoxDecoration(
                            image: DecorationImage(
                              image: AssetImage(tokens[index][3]),
                              fit: BoxFit.cover,
                            ),
                          ),
                        ),
                        title: Text(tokens[index][1] + ' ' + tokens[index][0]),
                        subtitle: Text('\$' + tokens[index][2]),
                        trailing: IconButton(icon: Icon(Icons.arrow_forward_ios),
                          onPressed: () {}),
                      );
                    }
                  ),
                  ListView.separated(
                    separatorBuilder: (BuildContext context, int index) => const Divider(),
                    itemCount: 10,
                    itemBuilder: (BuildContext context, int index) {
                      return Container(
                        child: Text('TODO ${index}'),
                      );
                    }
                  ),
                ],
                controller: _tabController!,
              ),
            ),
          ]
        )
      ),
    );
  }
}
