import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
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

//import 'package:esse/apps/assistant/models.dart';
//import 'package:esse/apps/assistant/provider.dart';

class DomainDetail extends StatefulWidget {
  const DomainDetail({Key? key}) : super(key: key);

  @override
  _DomainDetailState createState() => _DomainDetailState();
}

class _DomainDetailState extends State<DomainDetail> {
  bool _showProviders = false;
  bool _listHome = true;

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Scaffold(
      appBar: AppBar(
        title: Text(lang.domain + ' (${lang.wip})'),
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
            ? (this._showProviders ? _ListProviderScreen() : _ListNameScreen())
            : (this._showProviders ? _AddProviderScreen() : _RegisterScreen()),
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
  const _ListNameScreen({Key? key}) : super(key: key);

  Widget _nameItem(int id, String name, String provider, bool isActive, ColorScheme color) {
    return Card(
      elevation: 0.0,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(15)),
      margin: new EdgeInsets.symmetric(horizontal: 10.0, vertical: 8.0),
      child: Container(
        decoration: BoxDecoration(color: color.surface, borderRadius: BorderRadius.circular(15.0)),
        child: ListTile(
          contentPadding: EdgeInsets.symmetric(horizontal: 20.0, vertical: 10.0),
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
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;

    return Column(
      children: [
        _nameItem(0, "Sun", "domain.esse", true, color),
        _nameItem(0, "Huachuang", "domain.esse", false, color),
        _nameItem(0, "sun", "eth.esse", true, color),
      ]
    );
  }
}

class _ListProviderScreen extends StatelessWidget {
  const _ListProviderScreen({Key? key}) : super(key: key);

  Widget _providerItem(int id, String name, String address, bool isDefault, ColorScheme color, AppLocalizations lang) {
    return Card(
      elevation: 0.0,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(15)),
      margin: new EdgeInsets.symmetric(horizontal: 10.0, vertical: 8.0),
      child: Container(
        decoration: BoxDecoration(color: color.surface, borderRadius: BorderRadius.circular(15.0)),
        child: ListTile(
          contentPadding: EdgeInsets.symmetric(horizontal: 20.0, vertical: 10.0),
          leading: Container(
            padding: EdgeInsets.only(right: 12.0),
            decoration: new BoxDecoration(
              border: new Border(
                right: new BorderSide(width: 1.0, color: Color(0xA0ADB0BB)))),
            child: Icon(Icons.sync),
          ),
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
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Column(
      children: [
        _providerItem(0, "domain.esse", "0x89d240...77407b0e", true, color, lang),
        _providerItem(0, "eth.esse", "0x89d240...77407b0e", false, color, lang),
      ]
    );
  }
}

class _RegisterScreen extends StatefulWidget {
  const _RegisterScreen({Key? key}) : super(key: key);

  @override
  _RegisterScreenState createState() => _RegisterScreenState();
}

class _RegisterScreenState extends State<_RegisterScreen> {
  bool _showProviders = false;
  List _providers = [''];
  int _providerSelected = 0;

  TextEditingController _nameController = TextEditingController();
  TextEditingController _bioController = TextEditingController();

  FocusNode _nameFocus = FocusNode();
  FocusNode _bioFocus = FocusNode();

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    this._providers = ["domain.esse", "eth.esse", "btc.esse"];
    final maxIndex = this._providers.length - 1;

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
                    this._providers[this._providerSelected],
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
        const SizedBox(height: 20.0),
        ButtonText(action: () {}, text: lang.send),
      ]
    );
  }
}

class _AddProviderScreen extends StatefulWidget {
  const _AddProviderScreen({Key? key}) : super(key: key);

  @override
  _AddProviderScreenState createState() => _AddProviderScreenState();
}

class _AddProviderScreenState extends State<_AddProviderScreen> {
  TextEditingController _addrController = TextEditingController();
  FocusNode _addrFocus = FocusNode();

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
            text: lang.address,
            controller: _addrController,
            focus: _addrFocus),
        ),
        ButtonText(action: () {}, text: lang.send),
      ]
    );
  }
}
