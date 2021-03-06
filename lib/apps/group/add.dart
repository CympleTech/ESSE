import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/input_text.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/apps/group/models.dart';
import 'package:esse/apps/group/detail.dart';
import 'package:esse/provider.dart';
import 'package:esse/rpc.dart';

class GroupAddScreen extends StatefulWidget {
  final int fid;
  const GroupAddScreen({Key? key, required this.fid}) : super(key: key);

  @override
  _GroupAddScreenState createState() => _GroupAddScreenState();
}

class _GroupAddScreenState extends State<GroupAddScreen> {
  TextEditingController _nameController = TextEditingController();
  FocusNode _nameFocus = FocusNode();

  String _error = '';
  bool _waiting = false;

  @override
  void initState() {
    super.initState();
    _nameController.addListener(() {
        setState(() {
            this._error = '';
        });
    });
  }

  _send(String name, bool isDesktop) async {
    final res = await httpPost('group-create', [name]);
    if (res.isOk) {
      // use delayed because waiting this session added.
      Future.delayed(Duration(seconds: 1), () async {
          final fid = widget.fid;
          Navigator.pop(context);
          final sid = res.params[0];
          final id = res.params[1];
          final w = GroupChatDetail(id: id);
          rpc.send('group-member-join', [id, fid]);
          Provider.of<AccountProvider>(context, listen: false).updateActivedSession(sid);
          if (w != null) {
            if (isDesktop) {
              Provider.of<AccountProvider>(context, listen: false).updateActivedWidget(w);
            } else {
              Navigator.pop(context);
              Navigator.push(context, MaterialPageRoute(builder: (_) => w));
            }
          }
      });
    } else {
      setState(() {
          this._error = res.error;
          this._waiting = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);
    _nameFocus.requestFocus();

    return Column(
      children: [
        Text(this._error, style: TextStyle(color: Colors.red)),
        Container(
          padding: const EdgeInsets.only(bottom: 20.0, top: 10.0),
          child: InputText(
            enabled: !this._waiting,
            icon: Icons.account_circle,
            text: lang.groupChatName,
            controller: _nameController,
            focus: _nameFocus),
        ),
        ButtonText(
          enable: !this._waiting,
          text: this._waiting ? lang.waiting : lang.send,
          action: () {
            final name = _nameController.text.trim();
            if (name.length < 1) {
              return;
            }
            setState(() { this._waiting = true; });
            _send(name, isDesktop);
        }),
      ]
    );
  }
}
