import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/utils/pick_image.dart';
import 'package:esse/utils/pick_file.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/emoji.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/input_text.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/audio_recorder.dart';
import 'package:esse/widgets/show_contact.dart';
import 'package:esse/global.dart';
import 'package:esse/options.dart';
import 'package:esse/rpc.dart';

import 'package:esse/apps/domain/models.dart';

class DomainDetail extends StatefulWidget {
  const DomainDetail({Key? key}) : super(key: key);

  @override
  _DomainDetailState createState() => _DomainDetailState();
}

class _DomainDetailState extends State<DomainDetail> {
  bool _showProviders = false;
  bool _listHome = true;

  Map<int, ProviderServer> _providers = {};
  List<Name> _names = [];

  _domainList(List params) {
    this._providers.clear();
    params[0].forEach((param) {
        this._providers[param[0]] = ProviderServer.fromList(param);
    });
    this._names.clear();
    params[1].forEach((param) {
        this._names.add(Name.fromList(param));
    });
    setState(() {});
  }

  _domainProviderAdd(List params) {
    setState(() {
        this._listHome = true;
        this._showProviders = true;
        this._providers[params[0]] = ProviderServer.fromList(params);
    });
  }

  _domainRegisterSuccess(List params) {
    setState(() {
        this._listHome = true;
        this._showProviders = false;
        this._names.add(Name.fromList(params));
    });
  }

  @override
  void initState() {
    super.initState();

    // resigter rpc for current page.
    rpc.addListener('domain-list', _domainList, false);
    rpc.addListener('domain-provider-add', _domainProviderAdd, false);
    rpc.addListener('domain-register-success', _domainRegisterSuccess, false);

    rpc.send('domain-list', []);
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Scaffold(
      appBar: AppBar(
        title: Text(lang.domain),
        bottom: PreferredSize(
          child: Container(color: const Color(0x40ADB0BB), height: 1.0),
          preferredSize: Size.fromHeight(1.0)
        ),
        actions: [
          TextButton(
            onPressed: () {
              this._listHome = true;
              this._showProviders = !this._showProviders;
              setState(() {});
            },
            child: Padding(
              padding: const EdgeInsets.only(right: 10.0),
              child: Text(this._showProviders ? lang.domainShowName : lang.domainShowProvider)
            )
          ),
        ]
      ),
      body: Container(
        padding: const EdgeInsets.all(10.0),
        alignment: Alignment.topCenter,
        child: SingleChildScrollView(
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 20.0),
            child: this._listHome
            ? (this._showProviders ? _ListProviderScreen(this._providers) : _ListNameScreen(this._providers, this._names))
            : ((!this._showProviders && this._providers.length > 0)
              ? _RegisterScreen(this._providers.values.toList().asMap()) : _AddProviderScreen()
            ),
      ))),
      floatingActionButton: FloatingActionButton(
        onPressed: () => setState(() {
            this._listHome = !this._listHome;
          }
        ),
        child: Icon(this._listHome ? Icons.add : Icons.arrow_back, color: Colors.white),
        backgroundColor: Color(0xFF6174FF),
      ),
    );
  }
}

class _ListNameScreen extends StatelessWidget {
  final Map<int, ProviderServer> providers;
  final List<Name> names;

  const _ListNameScreen(this.providers, this.names);

  Widget _nameItem(int id, String name, String provider, bool isActive, ColorScheme color) {
    return Container(
      margin: const EdgeInsets.symmetric(vertical: 10.0),
      decoration: BoxDecoration(color: color.surface, borderRadius: BorderRadius.circular(15.0)),
      child: ListTile(
        contentPadding: EdgeInsets.symmetric(horizontal: 20.0, vertical: 8.0),
        leading: Container(
          padding: EdgeInsets.only(right: 12.0),
          decoration: new BoxDecoration(
            border: new Border(right: new BorderSide(width: 1.0, color: Color(0xA0ADB0BB)))),
          child: isActive ? Icon(Icons.toggle_on, color: color.primary) : Icon(Icons.toggle_off),
        ),
        title: Text(name, style: TextStyle(fontWeight: FontWeight.bold)),
        subtitle: Row(
          children: <Widget>[
            Expanded(child: Text(provider)),
          ],
        ),
        trailing: Icon(Icons.keyboard_arrow_right, size: 30.0),
      )
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;

    return Column(
      children: this.names.map(
        (name) => _nameItem(name.id, name.name, providers[name.provider]!.name, name.isActived, color)
      ).toList(),
    );
  }
}

class _ListProviderScreen extends StatelessWidget {
  final Map<int, ProviderServer> providers;

  const _ListProviderScreen(this.providers);

  Widget _providerItem(int id, String name, String address, bool isDefault, ColorScheme color, AppLocalizations lang) {
    return Container(
      margin: const EdgeInsets.symmetric(vertical: 10.0),
      decoration: BoxDecoration(color: color.surface, borderRadius: BorderRadius.circular(15.0)),
      child: ListTile(
        contentPadding: EdgeInsets.symmetric(horizontal: 20.0, vertical: 8.0),
        title: Text(name, style: TextStyle(fontWeight: FontWeight.bold)),
        subtitle: Row(
          children: <Widget>[
            Expanded(child: Text(address)),
          ],
        ),
        trailing: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (isDefault) Text(lang.default0, style: TextStyle(color: color.primary)),
            Icon(Icons.keyboard_arrow_right, size: 30.0),
          ]
        )
      )
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Column(
      children: this.providers.values.map(
        (provider) => _providerItem(provider.id, provider.name, addrPrint(provider.addr), provider.isDefault, color, lang)
      ).toList(),
    );
  }
}

class _RegisterScreen extends StatefulWidget {
  final Map<int, ProviderServer> providers;
  const _RegisterScreen(this.providers);

  @override
  _RegisterScreenState createState() => _RegisterScreenState();
}

class _RegisterScreenState extends State<_RegisterScreen> {
  bool _showProviders = false;
  int _providerSelected = -1;
  bool _registerNone = false;
  bool _waiting = false;

  TextEditingController _nameController = TextEditingController();
  TextEditingController _bioController = TextEditingController();

  FocusNode _nameFocus = FocusNode();
  FocusNode _bioFocus = FocusNode();

  _domainRegisterFailure(List _params) {
    setState(() {
        this._waiting = false;
        this._registerNone = true;
    });
  }

  @override
  void initState() {
    super.initState();
    rpc.addListener('domain-register-failure', _domainRegisterFailure, false);
  }


  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    if (this._providerSelected < 0) {
      this._providerSelected = 0;
      widget.providers.forEach((k, v) {
          if (v.isDefault) {
            this._providerSelected = k;
          }
      });
    }

    final maxIndex = widget.providers.length - 1;

    return Column(
      children: [
        Container(
          padding: EdgeInsets.symmetric(vertical: 10.0),
          height: 60.0,
          width: 600.0,
          child: Row(
            mainAxisSize: MainAxisSize.max,
            children: [
              TextButton(child: Icon(Icons.navigate_before),
                onPressed: this._providerSelected > 0 ? () => setState(() {
                    this._providerSelected = this._providerSelected - 1;
                }) : null,
              ),
              Expanded(
                child: Center(
                  child: Text(
                    widget.providers.length >= this._providerSelected
                    ? widget.providers[this._providerSelected]!.name
                    : '',
                    style: TextStyle(fontWeight: FontWeight.bold)),
              )),
              TextButton(child: Icon(Icons.navigate_next),
                onPressed: this._providerSelected < maxIndex ? () => setState(() {
                    this._providerSelected = this._providerSelected + 1;
                }) : null,
              ),
          ]
        )),
        Container(
          padding: EdgeInsets.symmetric(vertical: 10.0),
          child: InputText(
            icon: Icons.account_box,
            text: lang.domainName,
            controller: _nameController,
            focus: _nameFocus),
        ),
        Container(
          padding: EdgeInsets.symmetric(vertical: 10.0),
          child: InputText(
            icon: Icons.campaign,
            text: lang.bio,
            controller: _bioController,
            focus: _bioFocus),
        ),
        SizedBox(
          height: 40.0,
          child: Center(child: Text(this._registerNone ? lang.domainRegisterFailure : '',
              style: TextStyle(color: Colors.red))),
        ),
        ButtonText(
          enable: !this._waiting,
          action: () {
            String name = _nameController.text.trim();
            String bio = _bioController.text.trim();
            if (name.length > 0 && widget.providers.length >= this._providerSelected) {
              final provider = widget.providers[this._providerSelected]!.id;
              final addr = widget.providers[this._providerSelected]!.addr;
              rpc.send('domain-register', [provider, addr, name, bio]);
              setState(() {
                  this._waiting = true;
              });
            }
        }, text: this._waiting ? lang.waiting : lang.send),
      ]
    );
  }
}

class _AddProviderScreen extends StatefulWidget {
  const _AddProviderScreen();

  @override
  _AddProviderScreenState createState() => _AddProviderScreenState();
}

class _AddProviderScreenState extends State<_AddProviderScreen> {
  TextEditingController _addrController = TextEditingController();
  FocusNode _addrFocus = FocusNode();
  bool _waiting = false;

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Column(
      children: [
        Container(
          padding: EdgeInsets.symmetric(vertical: 10.0),
          child: Text(lang.domainAddProvider, style: TextStyle(fontWeight: FontWeight.bold)),
        ),
        Container(
          padding: EdgeInsets.symmetric(vertical: 30.0),
          child: InputText(
            icon: Icons.location_on,
            text: lang.address + ' (0x00..00)',
            controller: _addrController,
            focus: _addrFocus),
        ),
        ButtonText(
          enable: !this._waiting,
          action: () {
            final addr = addrParse(_addrController.text.trim());
            if (addr.length < 2) {
              return;
            }
            rpc.send('domain-provider-add', [addr]);
            setState(() {
                this._waiting = true;
            });
        }, text: this._waiting ? lang.waiting : lang.send),
      ]
    );
  }
}
