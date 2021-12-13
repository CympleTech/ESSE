import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:qr_flutter/qr_flutter.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/show_pin.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/input_text.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/default_core_show.dart';
import 'package:esse/provider.dart';
import 'package:esse/options.dart';
import 'package:esse/global.dart';
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

  Token _mainToken = Token();
  List<Token> _tokens = [];
  List<Transaction> _txs = [];

  @override
  void initState() {
    _tabController = new TabController(length: 2, vsync: this);

    rpc.addListener('wallet-generate', _walletGenerate, false);
    rpc.addListener('wallet-import', _walletGenerate, false);
    rpc.addListener('wallet-token', _walletToken, false);
    rpc.addListener('wallet-balance', _walletBalance, false);
    rpc.addListener('wallet-transfer', _walletTransfer, false);

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
      _changeAddress(address);
      setState(() {});
    }
  }

  _walletToken(List params) {
    final network = NetworkExtension.fromInt(params[0]);
    if (network == this._selectedNetwork!) {
      this._tokens.clear();
      params[1].forEach((param) {
          this._tokens.add(Token.fromList(param, '0'));
      });
    }
    setState(() {});
  }

  _walletBalance(List params) {
    final address = params[0];
    final network = NetworkExtension.fromInt(params[1]);
    if (address == this._selectedAddress!.address && network == this._selectedNetwork!) {
      final balance = params[2];

      if (params.length == 4) {
        final token = Token.fromList(params[3], balance);
        bool isNew = true;
        int key = 0;
        this._tokens.asMap().forEach((k, t) {
            if (t.contract == token.contract) {
              isNew = false;
              key = k;
            }
        });
        if (isNew) {
          this._tokens.add(token);
        } else {
          this._tokens[key].updateBalance(balance);
        }
      } else {
        this._mainToken = Token.eth(network);
        this._mainToken.updateBalance(balance);
      }
      setState(() {});
    }
  }

  _walletTransfer(List params) {
    final addressId = params[0];
    final network = NetworkExtension.fromInt(params[1]);
    final tx = Transaction.fromList(params[2]);

    bool isNew = true;
    this._txs.asMap().forEach((k, t) {
        if (t.hash == tx.hash) {
          isNew = false;
        }
    });
    if (isNew) {
      this._txs.add(tx);
      this._tabController!.animateTo(1);
    }
  }

  _load() async {
    final res = await httpPost(Global.httpRpc, 'wallet-list', []);
    if (res.isOk) {
      this._addresses.clear();
      res.params.forEach((param) {
          print(param);
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
      _changeNetwork(this._networks[0]);
    } else {
      rpc.send('wallet-token', [
          this._selectedNetwork!.toInt(), this._selectedAddress!.address
      ]);
    }
    this._mainToken = address.mainToken(this._selectedNetwork!);
  }

  _changeNetwork(Network network) {
    this._selectedNetwork = network;
    rpc.send('wallet-token', [
        this._selectedNetwork!.toInt(), this._selectedAddress!.address
    ]);
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
              final pin = context.read<AccountProvider>().activedAccount.pin;
              rpc.send('wallet-generate', [ChainToken.ETH.toInt(), pin]);
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

    List<PopupMenuEntry<int>> addressWidges = [];
    this._addresses.asMap().forEach((index, value) {
        addressWidges.add(_menuItem(index + 3, value, color, value == this._selectedAddress));
    });

    return Scaffold(
      appBar: AppBar(
        title: DropdownButton<Network>(
          icon: Container(),
          underline: Container(),
          value: this._selectedNetwork,
          onChanged: (Network? value) {
            if (value != null) {
              setState(() {
                  _changeNetwork(value);
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
            padding: const EdgeInsets.symmetric(horizontal: 20.0),
            child: PopupMenuButton<int>(
              child: Container(
                margin: const EdgeInsets.symmetric(vertical: 8.0, horizontal: 8.0),
                width: 40.0,
                decoration:  BoxDecoration(
                  color: color.surface,
                  borderRadius: BorderRadius.circular(25.0)
                ),
                child: Center(child: Text(this._selectedAddress!.icon()))
              ),
              onSelected: (int value) {
                if (value == 0) {
                  rpc.send('wallet-generate', [this._selectedAddress!.chain.toInt(), ""]);
                } else if (value == 1) {
                  showShadowDialog(context, Icons.vertical_align_bottom, lang.importAccount,
                    _ImportAccount(chain: this._selectedAddress!.chain), 20.0
                  );
                } else if (value == 2) {
                  //
                } else {
                  setState(() {
                      _changeAddress(this._addresses[value - 3]);
                  });
                }
              },
              itemBuilder: (context) {
                return addressWidges + <PopupMenuEntry<int>>[
                  PopupMenuItem<int>(
                    value: 0,
                    child: ListTile(
                      leading: Icon(Icons.add, color: const Color(0xFF6174FF)),
                      title: Text(lang.createAccount,
                        style: TextStyle(color: const Color(0xFF6174FF))),
                    ),
                  ),
                  PopupMenuItem<int>(
                    value: 1,
                    child: ListTile(
                      leading: Icon(Icons.vertical_align_bottom, color: const Color(0xFF6174FF)),
                      title: Text(lang.importAccount,
                        style: TextStyle(color: const Color(0xFF6174FF))),
                    ),
                  ),
                  PopupMenuItem<int>(
                    value: 2,
                    child: ListTile(
                      leading: Icon(Icons.settings, color: const Color(0xFF6174FF)),
                      title: Text(lang.setting,
                        style: TextStyle(color: const Color(0xFF6174FF))),
                    ),
                  )
                ];
              },
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
                        image: AssetImage(this._mainToken.logo),
                        fit: BoxFit.cover,
                      ),
                    ),
                  ),
                  Container(
                    height: 60.0,
                    alignment: Alignment.center,
                    child: Text(
                      "${this._mainToken.amount} ${this._mainToken.name}",
                      style: TextStyle(fontSize: 24.0, fontWeight: FontWeight.bold)),
                  ),
                  //Text('\$0.0', style: TextStyle(color: Color(0xFFADB0BB))),
                  const SizedBox(height: 8.0),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                    children: [
                      TextButton(
                        onPressed: () => showShadowDialog(
                          context, Icons.input, this._mainToken.name, _TransferToken(
                            chain: this._selectedAddress!.chain,
                            network: this._selectedNetwork!,
                            address: this._selectedAddress!,
                            token: this._mainToken,
                            addresses: this._addresses,
                          ), 0.0
                        ),
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 10.0),
                          decoration:  BoxDecoration(
                            color: Color(0xFF6174FF),
                            borderRadius: BorderRadius.circular(25.0)
                          ),
                          child: Center(child:
                            Row(
                              children: [
                                Icon(Icons.input, color: Colors.white, size: 18.0),
                                const SizedBox(width: 10.0),
                                Text(lang.send, style: TextStyle(color: Colors.white))
                              ]
                            )
                          )
                        )
                      ),
                      TextButton(
                        onPressed: () {
                          showShadowDialog(context, Icons.qr_code, lang.receive,
                            Column(
                              children: [
                                Container(
                                  width: 200.0,
                                  padding: const EdgeInsets.all(2.0),
                                  margin: const EdgeInsets.only(bottom: 20.0),
                                  decoration: BoxDecoration(
                                    borderRadius: BorderRadius.circular(5.0),
                                    border: Border.all(color: Color(0x40ADB0BB)),
                                    color: Colors.white,
                                  ),
                                  child: Center(
                                    child: QrImage(
                                      data: this._selectedAddress!.address,
                                      version: QrVersions.auto,
                                      foregroundColor: Colors.black,
                                    ),
                                  ),
                                ),
                                Text(this._selectedAddress!.address)
                              ]
                          ));
                        },
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 10.0),
                          decoration:  BoxDecoration(
                            color: Color(0xFF6174FF),
                            borderRadius: BorderRadius.circular(25.0)
                          ),
                          child: Center(child:
                            Row(
                              children: [
                                Icon(Icons.qr_code, color: Colors.white, size: 18.0),
                                const SizedBox(width: 10.0),
                                Text(lang.receive, style: TextStyle(color: Colors.white))
                              ]
                            )
                          )
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
                    itemCount: this._tokens.length + 1,
                    itemBuilder: (BuildContext context, int index) {
                      if (index == this._tokens.length) {
                        return TextButton(
                          child: Padding(
                            padding: const EdgeInsets.symmetric(vertical: 10.0),
                            child: Text('Add new Token' + ' ( ERC20 / ERC721 )')
                          ),
                          onPressed: () => showShadowDialog(
                            context, Icons.paid, 'Token', _ImportToken(
                              chain: this._selectedAddress!.chain,
                              network: this._selectedNetwork!,
                              address: this._selectedAddress!.address
                            ), 10.0
                          ),
                        );
                      } else {
                        final token = this._tokens[index];
                        return ListTile(
                          leading: Container(
                            width: 36.0,
                            height: 36.0,
                            decoration: BoxDecoration(
                              image: DecorationImage(
                                image: AssetImage(token.logo),
                                fit: BoxFit.cover,
                              ),
                            ),
                          ),
                          title: Text("${token.balance} ${token.name}",),
                          subtitle: Text(token.short()),
                          trailing: IconButton(icon: Icon(Icons.input, color: color.primary),
                            onPressed: () => showShadowDialog(
                              context, Icons.input, token.name, _TransferToken(
                                chain: this._selectedAddress!.chain,
                                network: this._selectedNetwork!,
                                address: this._selectedAddress!,
                                token: token,
                                addresses: this._addresses,
                              ), 0.0
                            ),
                          ),
                        );
                      }
                    }
                  ),
                  ListView.separated(
                    separatorBuilder: (BuildContext context, int index) => const Divider(),
                    itemCount: this._txs.length + 1,
                    itemBuilder: (BuildContext context, int index) {
                      if (index == this._txs.length) {
                        return TextButton(
                          child: Padding(
                            padding: const EdgeInsets.symmetric(vertical: 10.0),
                            child: Text(lang.loadMore)
                          ),
                          onPressed: () {
                            //
                          }
                        );
                      } else {
                        final tx = this._txs[index];
                        return ListTile(
                          title: Text('Hash: ' + tx.short_hash()),
                          subtitle: Text('To: ' + tx.short_to()),
                          trailing: IconButton(icon: Icon(Icons.link, color: color.primary),
                            onPressed: () {
                              launch(this._selectedNetwork!.txUrl() + tx.hash);
                            }
                          ),
                        );
                      }
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

  PopupMenuEntry<int> _menuItem(int value, Address address, ColorScheme color, bool selected) {
    return PopupMenuItem<int>(
      value: value,
      child: ListTile(
        leading: Icon(Icons.check, color: selected ? color.onSurface : Colors.transparent),
        title: Text(address.name),
        subtitle: Text(address.balance(this._selectedNetwork!) + ' ' + address.chain.symbol),
      ),
    );
  }
}

class _ImportAccount extends StatelessWidget {
  final ChainToken chain;
  TextEditingController _nameController = TextEditingController();
  FocusNode _nameFocus = FocusNode();

  _ImportAccount({Key? key, required this.chain}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    _nameFocus.requestFocus();

    return Column(
      children: [
        Container(
          padding: EdgeInsets.only(bottom: 20.0),
          child: InputText(
            icon: Icons.vpn_key,
            text: lang.secretKey,
            controller: _nameController,
            focus: _nameFocus),
        ),
        ButtonText(
          text: lang.send,
          action: () {
            final secret = _nameController.text.trim();
            if (secret.length < 32) {
              return;
            }
            rpc.send('wallet-import', [chain.toInt(), secret]);
            Navigator.pop(context);
        }),
      ]
    );
  }
}

class _ImportToken extends StatefulWidget {
  final ChainToken chain;
  final Network network;
  final String address;
  _ImportToken({Key? key, required this.chain, required this.network, required this.address}) : super(key: key);

  @override
  _ImportTokenState createState() => _ImportTokenState();
}

class _ImportTokenState extends State<_ImportToken> {
  TextEditingController _nameController = TextEditingController();
  FocusNode _nameFocus = FocusNode();
  ChainToken _selectedChain = ChainToken.ERC20;

  Widget _chain(ChainToken value, String show, color) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Radio(
          value: value,
          groupValue: _selectedChain,
          onChanged: (ChainToken? n) => setState(() {
              if (n != null) {
                _selectedChain = n;
              }
        })),
        _selectedChain == value
        ? Text(show, style: TextStyle(color: color.primary))
        : Text(show),
      ]
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final params = widget.network.params();
    _nameFocus.requestFocus();

    return Column(
      children: [
        Text(params[0], style: TextStyle(color: params[1], fontWeight: FontWeight.bold)),
        const SizedBox(height: 20.0),
        if (widget.chain.isEth())
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
          children: [
            _chain(ChainToken.ERC20, "ERC20", color),
            _chain(ChainToken.ERC721, "ERC721", color),
          ]
        ),
        const SizedBox(height: 10.0),
        Container(
          padding: EdgeInsets.only(bottom: 20.0),
          child: InputText(
            icon: Icons.location_on,
            text: lang.contract + ' (0x...)',
            controller: _nameController,
            focus: _nameFocus),
        ),
        ButtonText(
          text: lang.send,
          action: () {
            final contract = _nameController.text.trim();
            if (contract.length < 20) {
              return;
            }
            rpc.send('wallet-token-import', [
                _selectedChain.toInt(), widget.network.toInt(), widget.address, contract
            ]);
            Navigator.pop(context);
        }),
      ]
    );
  }
}

class _TransferToken extends StatefulWidget {
  final ChainToken chain;
  final Network network;
  final Address address;
  final Token token;
  final List addresses;
  _TransferToken({Key? key, required this.chain, required this.network, required this.address, required this.token, required this.addresses}) : super(key: key);

  @override
  _TransferTokenState createState() => _TransferTokenState();
}

class _TransferTokenState extends State<_TransferToken> {
  TextEditingController _nameController = TextEditingController();
  FocusNode _nameFocus = FocusNode();
  TextEditingController _amountController = TextEditingController();
  FocusNode _amountFocus = FocusNode();

  bool _myAccount = false;
  String _selectAddress = '';

  String _price = '';
  String _gas = '0';
  String _networkError = '';
  bool _checked = false;

  List<String> _nft = [];
  String _selectNft = '-';
  bool _nftInput = false;

  @override
  void initState() {
    super.initState();
    _nameController.addListener(() {
        setState(() {
            this._checked = false;
        });
    });
    _amountController.addListener(() {
        setState(() {
            this._checked = false;
        });
    });
    if (widget.token.isNft()) {
      _getNft();
    }
  }

  _getNft() async {
    final res = await httpPost(Global.httpRpc, 'wallet-nft', [
        widget.address.id, widget.token.id,
    ]);
    if (res.isOk) {
      final a = res.params[0];
      final t = res.params[1];
      res.params[2].forEach((hash) {
          this._nft.add(hash);
      });
      if (this._nft.length > 0) {
        this._selectNft = this._nft[0];
        this._nftInput = false;
      } else {
        this._nftInput = true;
      }
    } else {
      this._nftInput = true;
    }
    setState(() {});
  }

  _gasPrice(String to, String amount) async {
    final res = await httpPost(Global.httpRpc, 'wallet-gas-price', [
        widget.token.chain.toInt(), widget.network.toInt(),
        widget.address.address, to, amount,
        widget.token.contract
    ]);
    if (res.isOk) {
      this._price = unitBalance(res.params[0], 9, 0);
      this._gas = unitBalance(res.params[1], 18, 6);
      this._networkError = '';
      this._checked = true;
    } else {
      this._networkError = res.error;
    }

    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final params = widget.network.params();

    return Column(
      children: [
        Text(params[0], style: TextStyle(color: params[1], fontWeight: FontWeight.bold)),
        const SizedBox(height: 20.0),
        Container(
          margin: const EdgeInsets.only(bottom: 5.0),
          padding: const EdgeInsets.all(15.0),
          decoration: BoxDecoration(
            color: Color(0x266174FF),
            borderRadius: BorderRadius.circular(10.0)
          ),
          child: Column(
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  RichText(
                    text: TextSpan(
                      children: <TextSpan>[
                        TextSpan(
                          text: widget.address.name,
                          style: TextStyle(fontWeight: FontWeight.bold, color: color.primary)
                        ),
                        TextSpan(text: ' (' + widget.address.short() + ')', style: TextStyle(
                            fontSize: 14.0, fontStyle: FontStyle.italic, color: color.onSurface)),
                      ],
                    ),
                  )
                ]
              ),
              const SizedBox(height: 20.0),
              Text("${widget.token.balance} ${widget.token.name}",
                style: TextStyle(fontWeight: FontWeight.bold)),
              const SizedBox(height: 10.0),
              if (this._checked)
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(Icons.arrow_forward, color: Colors.green),
                  const SizedBox(width: 10.0),
                  (this._networkError.length > 1)
                  ? Text(this._networkError, style: TextStyle(color: Colors.red))
                  : RichText(
                    text: TextSpan(
                      text: 'Estimated Price = ',
                      style: TextStyle(
                        fontSize: 14.0, fontStyle: FontStyle.italic, color: Colors.green),
                      children: <TextSpan>[
                        TextSpan(text: this._price + ' Gwei',
                          style: TextStyle(fontWeight: FontWeight.bold)),
                        TextSpan(text: ', Gas ≈ '),
                        TextSpan(text: this._gas + ' ETH',
                          style: TextStyle(fontWeight: FontWeight.bold)),
                      ],
                    ),
                  )
                ]
              )
            ]
        )),
        this._myAccount
        ? Container(
          margin: const EdgeInsets.only(top: 22.0, bottom: 5.0),
          padding: const EdgeInsets.symmetric(horizontal: 20.0),
          decoration: BoxDecoration(
            color: color.surface,
            borderRadius: BorderRadius.circular(10.0)
          ),
          child: DropdownButtonHideUnderline(
            child: Theme(
              data: Theme.of(context).copyWith(
                canvasColor: color.background,
              ),
              child: DropdownButton<String>(
                iconEnabledColor: Color(0xFFADB0BB),
                isExpanded: true,
                value: this._selectAddress,
                onChanged: (String? addr) {
                  if (addr != null) {
                    setState(() {
                        this._checked = false;
                        this._selectAddress = addr;
                    });
                  }
                },
                items: widget.addresses.map((address) {
                    return DropdownMenuItem<String>(
                      value: address.address,
                      child: Row(
                        children: [
                          Expanded(
                            child: Text("${address.name}",
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: TextStyle(fontSize: 16)
                            ),
                          ),
                          Text(" (${address.short()})", style: TextStyle(fontSize: 16)),
                          const SizedBox(width: 10.0),
                        ]
                      ),
                    );
                }).toList(),
              ),
            ),
          )
        )
        : Container(
          padding: const EdgeInsets.only(top: 20.0, bottom: 5.0),
          child: Row(
            children: [
              Expanded(
                child: InputText(
                  icon: Icons.person,
                  text: lang.account.toLowerCase() + ' (0x...)',
                  controller: _nameController,
                  focus: _nameFocus
              )),
              SizedBox(width: 80.0,
                child: IconButton(icon: Icon(Icons.qr_code, color: color.primary),
                  onPressed: () {
                    //
                })
              )
          ])
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.start,
          children: [
            const SizedBox(width: 10.0),
            TextButton(child: Text(
                this._myAccount ? 'Input Account' : 'Between Accounts'
              ), onPressed: () => setState(() {
                  this._myAccount = !this._myAccount;
                  if (this._selectAddress.length == 0) {
                    this._selectAddress = widget.addresses[0].address;
                  }
              })
            ),
          ]
        ),
        widget.token.isNft()
        ? Container(
          padding: const EdgeInsets.only(top: 15.0, bottom: 20.0),
          child: Row(
            children: [
              Expanded(
                child: this._nftInput
                ? InputText(
                  icon: Icons.verified,
                  text: '0xAa..',
                  controller: _amountController,
                  focus: _amountFocus)
                : DropdownButtonHideUnderline(
                  child: Theme(
                    data: Theme.of(context).copyWith(
                      canvasColor: color.background,
                    ),
                    child: DropdownButton<String>(
                      iconEnabledColor: Color(0xFFADB0BB),
                      isExpanded: true,
                      value: this._selectNft,
                      onChanged: (String? value) {
                        if (value != null) {
                          setState(() {
                              this._selectNft = value;
                          });
                        }
                      },
                      items: this._nft.map((value) {
                          return DropdownMenuItem<String>(
                            value: value,
                            child: Text(value, maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: TextStyle(fontSize: 16)
                            ),
                          );
                      }).toList(),
                    ),
                  ),
                )
              ),
              SizedBox(width: 80.0,
                child: IconButton(icon: this._nftInput ? Icon(Icons.fact_check, color: color.primary) : Icon(Icons.edit, color: Colors.green),
                  onPressed: () => setState(() {
                      this._nftInput = !this._nftInput;
                  })
                ),
              )
            ]
          )
        )
        : Container(
          padding: const EdgeInsets.only(top: 15.0, bottom: 20.0),
          child: Row(
            children: [
              Expanded(
                child: InputText(
                  icon: Icons.paid,
                  text: '0.0',
                  controller: _amountController,
                  focus: _amountFocus),
              ),
              SizedBox(width: 80.0,
                child: IconButton(icon: Text('Max', style: TextStyle(color: color.primary)),
                  onPressed: () => setState(() {
                      if (widget.token.chain.isMain()) {
                        final a = widget.token.amount - double.parse(this._gas);
                        _amountController.text = "${a}";
                      } else {
                        _amountController.text = "${widget.token.amount}";
                      }
                  })
                ),
              )
            ]
          )
        ),
        this._checked
        ? ButtonText(
          text: lang.send,
          action: () {
            String to = _nameController.text.trim();
            if (_myAccount) {
              to = this._selectAddress;
            }
            final a = _amountController.text.trim();
            if (double.parse(a) == 0) {
              _amountFocus.requestFocus();
              return;
            }
            if (to.length < 20) {
              _nameFocus.requestFocus();
              return;
            }
            final amount = restoreBalance(a, widget.token.decimal);
            final gid = context.read<AccountProvider>().activedAccount.gid;
            showShadowDialog(
              context,
              Icons.security_rounded,
              lang.verifyPin,
              PinWords(
                gid: gid,
                callback: (key) async {
                  Navigator.of(context).pop();
                  rpc.send('wallet-transfer', [
                      widget.token.chain.toInt(), widget.network.toInt(),
                      widget.address.id, to, restoreBalance(amount, widget.token.decimal),
                      widget.token.contract, key,
                  ]);
                  Navigator.of(context).pop();
              }),
              0.0,
            );
        })
        : ButtonText(
          text: lang.check,
          action: () {
            String to = _nameController.text.trim();
            if (_myAccount) {
              to = this._selectAddress;
            }
            final a = _amountController.text.trim();
            if (a.length == 0 || double.parse(a) == 0) {
              _amountFocus.requestFocus();
              return;
            }
            if (to.length < 20) {
              _nameFocus.requestFocus();
              return;
            }
            final amount = restoreBalance(a, widget.token.decimal);
            _gasPrice(to, amount);
        })
      ]
    );
  }
}
