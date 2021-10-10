import 'package:flutter/material.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/rpc.dart';
import 'package:esse/apps/domain/models.dart';

class AccountDomainScreen extends StatefulWidget {
  final String name;

  const AccountDomainScreen({Key? key, required this.name}) : super(key: key);

  @override
  AccountDomainScreenState createState() => AccountDomainScreenState();
}

class AccountDomainScreenState extends State<AccountDomainScreen> {
  Map<int, ProviderServer> _providers = {};

  int _selected = -1;
  bool _showName = false;
  bool _exist = false;
  bool _waiting = false;

  TextEditingController _nameController = new TextEditingController();
  FocusNode _nameFocus = new FocusNode();

  _domainList(List params) {
    this._providers.clear();
    params[0].forEach((param) {
        this._providers[param[0]] = ProviderServer.fromList(param);
    });
    setState(() {});
  }

  _domainRegisterSuccess(List params) {
    // TODO toast.
    Navigator.of(context).pushNamedAndRemoveUntil("/", (Route<dynamic> route) => false);
  }

  _domainRegisterFailure(List _params) {
    setState(() {
        this._waiting = false;
        this._exist = true;
        this._showName = true;
    });
  }

  @override
  initState() {
    super.initState();

    rpc.addListener('domain-list', _domainList, false);
    rpc.addListener('domain-register-success', _domainRegisterSuccess, false);
    rpc.addListener('domain-register-failure', _domainRegisterFailure, false);
    rpc.send('domain-list', []);

    _nameController.text = widget.name;
    _nameFocus.addListener(() {
        setState(() {});
    });
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    if (this._selected < 0 && this._providers.length > 0) {
      this._selected = 0;
      this._providers.forEach((k, v) {
          if (v.isDefault) {
            this._selected = k;
          }
      });
    }

    final provider = this._providers.length > 0
    ? this._providers[this._selected]! : ProviderServer.empty();

    return Scaffold(
      body: Padding(
        padding: const EdgeInsets.all(20.0),
        child: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              RichText(
                text: TextSpan(
                  text: lang.register + ' ',
                  style: TextStyle(
                    color: color.onSurface, fontSize: 20.0, fontWeight: FontWeight.bold
                  ),
                  children: <TextSpan>[
                    TextSpan(text: widget.name, style: TextStyle(color: Color(0xFF6174FF))),
                    TextSpan(text: '  ->  '),
                    TextSpan(text: provider.name,
                      style: TextStyle(color: Color(0xFF6174FF), fontStyle: FontStyle.italic)
                    ),
                    TextSpan(text: ' ?'),
                  ],
                ),
              ),
              const SizedBox(height: 10.0),
              Container(
                width: 600.0,
                child: Text(lang.domainCreateTip),
              ),
              SizedBox(
                height: 40.0,
                child: Center(child: Text(this._exist ? lang.domainRegisterFailure : '',
                    style: TextStyle(color: Colors.red))),
              ),
              if (this._showName)
              Container(
                width: 600.0,
                height: 50.0,
                padding: const EdgeInsets.symmetric(horizontal: 20.0),
                decoration: BoxDecoration(
                  color: color.surface,
                  border: Border.all(
                    color: _nameFocus.hasFocus ? color.primary : color.surface),
                  borderRadius: BorderRadius.circular(10.0),
                ),
                child: TextField(
                  style: TextStyle(fontSize: 16.0),
                  decoration: InputDecoration(
                    border: InputBorder.none,
                    hintText: lang.domainName,
                  ),
                  controller: _nameController,
                  focusNode: _nameFocus
                ),
              ),
              const SizedBox(height: 20.0),
              ButtonText(
                enable: this._providers.length > 0 && !this._waiting,
                text: this._waiting ? lang.waiting : lang.send,
                action: () {
                  final name = _nameController.text.trim();
                  if (name.length > 0) {
                    rpc.send('domain-register', [provider.id, provider.addr, name, 'Hello ESSE!']);
                    setState(() {
                        this._waiting = true;
                        this._exist = false;
                    });
                  }
              }),
              const SizedBox(height: 20.0),
              InkWell(
                child: Container(
                  width: 600.0,
                  height: 50.0,
                  decoration: BoxDecoration(
                    border: Border.all(color: Color(0xFF6174FF)),
                    borderRadius: BorderRadius.circular(10.0)),
                  child: Center(child: Text(lang.skip, style: TextStyle(
                        fontSize: 20.0, color: Color(0xFF6174FF)
                  ))),
                ),
                onTap: () {
                  Navigator.of(context).pushNamedAndRemoveUntil("/", (Route<dynamic> route) => false);
                }
              ),
            ]
          )
        )
      )
    );
  }
}
