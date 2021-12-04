import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/input_text.dart';
import 'package:esse/global.dart';
import 'package:esse/options.dart';
import 'package:esse/rpc.dart';

class WalletDetail extends StatefulWidget {
  const WalletDetail({Key? key}) : super(key: key);

  @override
  _WalletDetailState createState() => _WalletDetailState();
}

class Network {
  const Network(this.name, this.color);
  final Color color;
  final String name;
}

class _WalletDetailState extends State<WalletDetail> with SingleTickerProviderStateMixin {
  TabController? _tabController;

  List tokens = [
    ['ETH', '100', '1000', 'assets/logo/logo_eth.png'],
    ['USDT', '2000', '2000', 'assets/logo/logo_tether.png'],
    ['XXX', '100', '1000', 'assets/logo/logo_erc20.png'],
    ['FFF', '100', '1000', 'assets/logo/logo_erc20.png'],
  ];

  Network? selectedNetwork;
  List<Network> networks = <Network>[
    const Network(
      'Ethereum Mainnet',
      const Color(0xFF167F67)
    ),
    const Network(
      'Ropsten Test Network',
      Colors.orange,
    ),
    const Network(
      'Rinkeby Test Network',
      Colors.orange,
    ),
    const Network(
      'Localhost 8545',
      const Color(0xFF6174FF),
    ),
  ];

  @override
  void initState() {
    _tabController = new TabController(length: 2, vsync: this);
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Scaffold(
      appBar: AppBar(
        title: DropdownButton<Network>(
          icon: Container(),
          underline: Container(),
          hint:  Text("Select network"),
          value: selectedNetwork,
          onChanged: (Network? value) {
            if (value != null) {
              setState(() {
                  selectedNetwork = value;
              });
            }
          },
          items: networks.map((Network network) {
              return  DropdownMenuItem<Network>(
                value: network,
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 6.0),
                  decoration:  BoxDecoration(
                    border: Border.all(width: 1.0, color: network.color),
                    borderRadius: BorderRadius.circular(25.0)
                  ),
                  child: Row(
                    children: <Widget>[
                      Icon(Icons.public, color: network.color, size: 18.0),
                      const SizedBox(width: 10),
                      Text(network.name, style: TextStyle(color: network.color, fontSize: 14.0)),
                    ],
                )),
              );
          }).toList(),
        ),
        actions: [
          TextButton(
            onPressed: () {
              setState(() {});
            },
            child: Container(
              margin: const EdgeInsets.symmetric(horizontal: 10.0),
              width: 40.0,
              height: 40.0,
              decoration:  BoxDecoration(
                color: color.surface,
                borderRadius: BorderRadius.circular(25.0)
              ),
              child: Center(child: Text('A'))
            )
          ),
        ]
      ),
      body: Container(
        alignment: Alignment.topCenter,
        padding: const EdgeInsets.symmetric(horizontal: 20.0),
        child: Column(
          children:[
            InkWell(
              onTap: () {
                //Clipboard.setData(ClipboardData(text: gidText(widget.id)));
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
                    Text('Account1', style: TextStyle(fontSize: 18.0)),
                    const SizedBox(height: 4.0),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Text('0x222...334444', style: TextStyle(color: Color(0xFFADB0BB))),
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
