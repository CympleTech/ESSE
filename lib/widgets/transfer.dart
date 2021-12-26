import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/show_pin.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/input_text.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/apps/wallet/models.dart';
import 'package:esse/provider.dart';
import 'package:esse/rpc.dart';

class Transfer extends StatefulWidget {
  final Function callback;
  final String to;
  const Transfer({Key? key, required this.callback, required this.to}) : super(key: key);

  @override
  _TransferState createState() => _TransferState();
}

class _TransferState extends State<Transfer> {
  Network _selectedNetwork = Network.EthMain;
  Color _networkColor = Colors.green;
  List<Network> _networks = [];

  Address? _selectedAddress;
  List<Address> _addresses = [];

  Token _selectedToken = Token.eth(Network.EthMain);
  Token _mainToken = Token.eth(Network.EthMain);
  List<Token> _tokens = [];
  List<String> _nft = [];
  String _selectedNft = '';

  TextEditingController _amountController = TextEditingController();
  FocusNode _amountFocus = FocusNode();

  bool _checked = false;
  bool _checking = false;
  String _price = '';
  String _gas = '0';
  String _networkError = '';

  Map<ChainToken, String> _mains = {};
  String _main = '';
  ChainToken _mainChain = ChainToken.ETH;

  @override
  initState() {
    rpc.addListener('wallet-token', _walletToken);
    rpc.addListener('wallet-balance', _walletBalance);
    _amountController.addListener(() {
        setState(() {
            this._checked = false;
            this._checking = false;
        });
    });
    super.initState();
    _loadWallet();
    _split_address(widget.to);
  }

  _split_address(String s) {
    print(s);
    if (s.length > 0) {
      s.split(",").forEach((ss) {
          final sss = ss.split(":");
          this._mains[ChainTokenExtension.fromInt(int.parse(sss[0]))] = sss[1];
      });
      if (this._mains.length == 0) {
        Navigator.of(context).pop();
      }
      this._mainChain = ChainToken.ETH;
      this._main = this._mains[ChainToken.ETH]!;
      setState(() {});
    }
  }

  _walletToken(List params) {
    final network = NetworkExtension.fromInt(params[0]);
    if (network == this._selectedNetwork) {
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
    if (address == this._selectedAddress!.address && network == this._selectedNetwork) {
      final balance = params[2];

      if (params.length == 4) {
        for (int i=0;i<this._tokens.length;i++) {
          if (this._tokens[i].contract == params[3][4]) {
            this._tokens[i].updateBalance(balance);
          }
        }
      } else {
        this._mainToken.updateBalance(balance);
      }
      setState(() {});
    }
  }

  _loadWallet() async {
    final res = await httpPost('wallet-list', []);
    if (res.isOk) {
      this._addresses.clear();
      res.params.forEach((param) {
          final address = Address.fromList(param);
          this._addresses.add(address);
          if (address.isMain && this._mainChain == address.chain) {
            _changeAddress(address);
          }
      });
      setState(() {});
    } else {
      print(res.error);
    }
  }

  _changeAddress(Address address) {
    if (!this._mains.containsKey(address.chain)) {
      return;
    }
    this._main = this._mains[address.chain]!;
    this._mainChain = address.chain;
    this._selectedAddress = address;
    this._networks = address.networks();
    if (!this._networks.contains(this._selectedNetwork)) {
      _changeNetwork(this._networks[0]);
    } else {
      rpc.send('wallet-token', [
          this._selectedNetwork.toInt(), this._selectedAddress!.address
      ]);
    }
    this._mainToken = address.mainToken(this._selectedNetwork);
    this._selectedToken = _mainToken;
  }

  _changeNetwork(Network network) {
    this._selectedNetwork = network;
    this._networkColor = network.params()[1];
    rpc.send('wallet-token', [
        this._selectedNetwork.toInt(), this._selectedAddress!.address
    ]);
  }

  _gasPrice(String amount) async {
    if (this._main.length < 2) {
      return;
    }
    final res = await httpPost('wallet-gas-price', [
        this._selectedToken.chain.toInt(), this._selectedNetwork.toInt(),
        this._selectedAddress!.address, this._main, amount,
        this._selectedToken.contract
    ]);
    if (res.isOk) {
      this._price = unitBalance(res.params[0], 9, 0);
      this._gas = unitBalance(res.params[1], 18, 6);
      this._networkError = '';
      this._checked = true;
    } else {
      this._networkError = res.error;
    }
    this._checking = false;
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    if (_selectedAddress == null) {
      return Center(child: Text(lang.waiting));
    }

    final mainWidget = [DropdownMenuItem<Token>(
        value: this._mainToken,
        child: Row(
          children: [
            Text(this._mainToken.name),
            Spacer(),
            Text(this._selectedAddress!.balance(this._selectedNetwork)),
            const SizedBox(width: 10.0),
          ]
        ),
    )];

    return Column(
      children: [
        Text(this._main, style: TextStyle(color: color.primary)),
        const SizedBox(height: 10.0),
        Row(
          children: [
            Icon(Icons.public, color: this._networkColor),
            const SizedBox(width: 10.0),
            Expanded(
              child: DropdownButtonHideUnderline(
                child: DropdownButton<Network>(
                  iconEnabledColor: Color(0xFFADB0BB),
                  isExpanded: true,
                  value: this._selectedNetwork,
                  onChanged: (Network? network) {
                    if (network != null) {
                      setState(() {
                          this._checked = false;
                          _changeNetwork(network);
                      });
                    }
                  },
                  items: this._networks.map((network) {
                      final params = network.params();
                      return DropdownMenuItem<Network>(
                        value: network,
                        child: Text(params[0], style: TextStyle(fontSize: 16, color: params[1]))
                      );
                  }).toList(),
                ),
              )
        )]), // network select.
        Row(
          children: [
            const Icon(Icons.account_circle, color: Color(0xFF6174FF)),
            const SizedBox(width: 10.0),
            Expanded(
              child: DropdownButtonHideUnderline(
                child: DropdownButton<Address>(
                  iconEnabledColor: Color(0xFFADB0BB),
                  isExpanded: true,
                  value: this._selectedAddress,
                  onChanged: (Address? addr) {
                    if (addr != null) {
                      setState(() {
                          this._checked = false;
                          _changeAddress(addr);
                      });
                    }
                  },
                  items: this._addresses.map((address) {
                      return DropdownMenuItem<Address>(
                        value: address,
                        child: Row(
                          children: [
                            Text(address.name),
                            Spacer(),
                            Text('(' + address.short()  + ')'),
                            const SizedBox(width: 10.0),
                          ]
                        ),
                      );
                  }).toList(),
                ),
              ),
        )]), // address select.
        Row(
          children: [
            const SizedBox(width: 2.0),
            Container(
              width: 20.0,
              height: 20.0,
              decoration: BoxDecoration(
                image: DecorationImage(
                  image: AssetImage(this._selectedToken.logo),
                  fit: BoxFit.cover,
                ),
              ),
            ),
            const SizedBox(width: 10.0),
            Expanded(
              child: DropdownButtonHideUnderline(
                child: DropdownButton<Token>(
                  iconEnabledColor: Color(0xFFADB0BB),
                  isExpanded: true,
                  value: this._selectedToken,
                  onChanged: (Token? token) {
                    if (token != null) {
                      setState(() {
                          this._checked = false;
                          this._selectedToken = token;
                      });
                    }
                  },
                  items: mainWidget + this._tokens.map((token) {
                      return DropdownMenuItem<Token>(
                        value: token,
                        child: Row(
                          children: [
                            Text(token.name),
                            Spacer(),
                            Text(token.balance),
                            const SizedBox(width: 10.0),
                          ]
                        ),
                      );
                  }).toList(),
                ),
              ),
        )]), // token select.
        const SizedBox(height: 20.0),
        InputText(
          icon: this._selectedToken.isNft() ? Icons.verified : Icons.paid,
          text: this._selectedToken.isNft() ? 'TokenID' : '0.0',
          controller: _amountController,
          focus: _amountFocus
        ),
        const SizedBox(height: 10.0),
        if (this._checked)
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.arrow_forward, color: Colors.green),
            const SizedBox(width: 10.0),
            Expanded(
              child: this._networkError.length > 1
              ? Text(this._networkError,
                textAlign: TextAlign.center, style: TextStyle(color: Colors.red))
              : RichText(
                textAlign: TextAlign.center,
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
            ),
          ]
        ),
        const SizedBox(height: 10.0),
        this._checked
        ? ButtonText(
          text: lang.send,
          action: () {
            String a = _amountController.text.trim();
            if (this._selectedToken.isNft()) {
              a = this._selectedNft;
            }
            if (a.length == 0 || (!this._selectedToken.isNft() && double.parse(a) == 0)) {
              _amountFocus.requestFocus();
              return;
            }
            final amount = restoreBalance(a, this._selectedToken.decimal);
            final gid = context.read<AccountProvider>().activedAccount.gid;
            showShadowDialog(
              context,
              Icons.security_rounded,
              lang.verifyPin,
              PinWords(
                gid: gid,
                callback: (key) async {
                  if (this._main.length < 2) {
                    return;
                  }
                  Navigator.of(context).pop();
                  final res = await httpPost('wallet-transfer', [
                      this._selectedToken.chain.toInt(), this._selectedNetwork.toInt(),
                      this._selectedAddress!.id, this._main, amount,
                      this._selectedToken.contract, key,
                  ]);
                  if (res.isOk) {
                    final addressId = res.params[0];
                    final network = NetworkExtension.fromInt(res.params[1]);
                    final tx = Transaction.fromList(res.params[2]);
                    widget.callback(tx.hash, tx.to, amount,
                      this._selectedToken.name, this._selectedNetwork.toInt(),
                      this._selectedToken.decimal,
                    );
                    Navigator.of(context).pop();
                  } else {
                    this._networkError = res.error;
                  }
              }),
              0.0,
            );
        })
        : ButtonText(
          enable: !this._checking,
          text: this._checking ? lang.waiting : lang.check,
          action: () {
            String a = _amountController.text.trim();
            if (this._selectedToken.isNft()) {
              a = this._selectedNft;
            }
            if (a.length == 0 || (!this._selectedToken.isNft() && double.parse(a) == 0)) {
              _amountFocus.requestFocus();
              return;
            }
            final amount = restoreBalance(a, this._selectedToken.decimal);
            _gasPrice(amount);
            setState(() {
                this._checking = true;
            });
        })
    ]);
  }
}
