import 'dart:convert';

import 'package:crypto/crypto.dart';
import 'package:flutter/material.dart';

import 'package:esse/l10n/localizations.dart';

const pinLength = 6;

Widget _circle(
    bool isError, bool filled, Color filledColor, Color borderColor) {
  return Container(
    width: 20.0,
    height: 20.0,
    decoration: BoxDecoration(
        color: filled ? filledColor : Colors.transparent,
        shape: BoxShape.circle,
        border: Border.all(color: isError ? Colors.red : borderColor, width: 2.0)),
  );
}

Widget _keyboradInput(Color color, Color bg, String text, Function callback) {
  return Container(
    margin: EdgeInsets.all(5.0),
    child: Ink(
        decoration: BoxDecoration(
          color: bg,
          borderRadius: BorderRadius.circular(15.0),
          boxShadow: [
            BoxShadow(
              color: Color(0x40ADB0BB),
              spreadRadius: 0.0,
              blurRadius: 20.0,
              offset: Offset(0, 10),
            ),
          ],
        ),
        child: InkResponse(
          borderRadius: BorderRadius.circular(15.0),
          highlightShape: BoxShape.rectangle,
          hoverColor: Color(0x40ADB0BB).withOpacity(0.1),
          splashColor: Color(0x40ADB0BB),
          containedInkWell: true,
          onTap: () => callback(text),
          child: Container(
            padding: EdgeInsets.all(20.0),
            child: Container(
                width: 20.0,
                height: 20.0,
                alignment: Alignment.center,
                child:
                    Text(text, style: TextStyle(color: color, fontSize: 18.0))),
          ),
        )),
  );
}

class PinWords extends StatefulWidget {
  final Function callback;
  final String hashPin;

  PinWords({Key? key, required this.hashPin, required this.callback}) : super(key: key);

  @override
  _PinWordsState createState() => _PinWordsState();
}

class _PinWordsState extends State<PinWords> {
  String pinWords = '';
  bool isError = false;

  _inputCallback(String text) {
    isError = false;
    pinWords += text;
    if (pinWords.length < pinLength) {
      setState(() {});
    } else {
      final bytes = utf8.encode(pinWords);
      final lock = "${sha256.convert(bytes)}";

      if (widget.hashPin != "" && widget.hashPin != lock) {
        setState(() {
          pinWords = '';
          isError = true;
        });
      } else {
        setState(() {});
        widget.callback(pinWords, lock);
      }
    }
  }

  List<Widget> _buildCircles(ColorScheme color) {
    var list = <Widget>[];
    for (int i = 0; i < pinLength; i++) {
      list.add(
        Container(
          margin: EdgeInsets.all(6.0),
          child: _circle(
            isError,
            i < pinWords.length,
            color.primary,
            color.primaryVariant,
          ),
        ),
      );
    }
    return list;
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    //final lang = AppLocalizations.of(context);

    return Column(children: [
      Container(
        margin: const EdgeInsets.only(bottom: 20.0),
        height: 40,
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: _buildCircles(color),
        ),
      ),
      Row(mainAxisAlignment: MainAxisAlignment.center, children: [
        _keyboradInput(color.primary, color.background, '1', _inputCallback),
        _keyboradInput(color.primary, color.background, '2', _inputCallback),
        _keyboradInput(color.primary, color.background, '3', _inputCallback),
      ]),
      Row(mainAxisAlignment: MainAxisAlignment.center, children: [
        _keyboradInput(color.primary, color.background, '4', _inputCallback),
        _keyboradInput(color.primary, color.background, '5', _inputCallback),
        _keyboradInput(color.primary, color.background, '6', _inputCallback),
      ]),
      Row(mainAxisAlignment: MainAxisAlignment.center, children: [
        _keyboradInput(color.primary, color.background, '7', _inputCallback),
        _keyboradInput(color.primary, color.background, '8', _inputCallback),
        _keyboradInput(color.primary, color.background, '9', _inputCallback),
      ]),
      Row(mainAxisAlignment: MainAxisAlignment.center, children: [
        _keyboradInput(color.primary, color.background, '0', _inputCallback),
        GestureDetector(
          onTap: () {
            if (pinWords.length > 0) {
              setState(() {
                pinWords = pinWords.substring(0, pinWords.length - 1);
              });
            }
          },
          child: Container(
            padding: EdgeInsets.all(20.0),
            margin: EdgeInsets.all(5.0),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(15.0),
              color: Color(0xFF6174FF),
              boxShadow: [
                BoxShadow(
                  color: Color(0x2A6174FF),
                  spreadRadius: 0.0,
                  blurRadius: 10.0,
                  offset: Offset(0, 10),
                ),
              ],
            ),
            child: Container(
                width: 90.0,
                height: 20.0,
                alignment: Alignment.center,
                child: Icon(Icons.backspace_rounded,
                    size: 20.0, color: Colors.white)),
          ),
        ),
      ])
    ]);
  }
}

class SetPinWords extends StatefulWidget {
  final Function callback;

  SetPinWords({Key? key, required this.callback}) : super(key: key);

  @override
  _SetPinWordsState createState() => _SetPinWordsState();
}

class _SetPinWordsState extends State<SetPinWords> {
  bool _first = true;
  String _pinWords = '';
  String _repeatPinWords = '';
  bool _isError = false;

  _inputCallback(String text) async {
    this._isError = false;
    this._pinWords += text;
    if (this._pinWords.length < pinLength) {
      setState(() {});
    } else {
      if (_first) {
        setState(() {});
        new Future.delayed(Duration(milliseconds: 200),
          () => setState(() {
              this._repeatPinWords = this._pinWords;
              this._pinWords = '';
              this._first = false;
        }));
      } else {
        if (this._pinWords != this._repeatPinWords) {
          setState(() {
              this._isError = true;
              this._first = true;
              this._repeatPinWords = '';
              this._pinWords = '';
          });
        } else {
          setState(() {});
          final bytes = utf8.encode(this._pinWords);
          final lock = "${sha256.convert(bytes)}";
          widget.callback(this._pinWords, lock);
        }
      }
    }
  }

  List<Widget> _buildCircles(ColorScheme color) {
    var list = <Widget>[];
    for (int i = 0; i < pinLength; i++) {
      list.add(
        Container(
          margin: EdgeInsets.all(6.0),
          child: _circle(
            this._isError,
            i < this._pinWords.length,
            color.primary,
            color.primaryVariant,
          ),
        ),
      );
    }
    return list;
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Column(children: [
      Container(
        height: 20.0,
        child: this._isError
        ? Text(lang.repeatPin, style: TextStyle(fontSize: 14.0, color: Colors.red))
        : Text(this._first ? ' ' : lang.repeatPin,
          style: TextStyle(fontSize: 14.0, color: color.primary)),
      ),
      Container(
        margin: const EdgeInsets.only(bottom: 16.0),
        height: 40,
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: _buildCircles(color),
        ),
      ),
      Row(mainAxisAlignment: MainAxisAlignment.center, children: [
        _keyboradInput(color.primary, color.background, '1', _inputCallback),
        _keyboradInput(color.primary, color.background, '2', _inputCallback),
        _keyboradInput(color.primary, color.background, '3', _inputCallback),
      ]),
      Row(mainAxisAlignment: MainAxisAlignment.center, children: [
        _keyboradInput(color.primary, color.background, '4', _inputCallback),
        _keyboradInput(color.primary, color.background, '5', _inputCallback),
        _keyboradInput(color.primary, color.background, '6', _inputCallback),
      ]),
      Row(mainAxisAlignment: MainAxisAlignment.center, children: [
        _keyboradInput(color.primary, color.background, '7', _inputCallback),
        _keyboradInput(color.primary, color.background, '8', _inputCallback),
        _keyboradInput(color.primary, color.background, '9', _inputCallback),
      ]),
      Row(mainAxisAlignment: MainAxisAlignment.center, children: [
        _keyboradInput(color.primary, color.background, '0', _inputCallback),
        GestureDetector(
          onTap: () {
            if (this._pinWords.length > 0) {
              setState(() {
                this._pinWords = this._pinWords.substring(0, this._pinWords.length - 1);
              });
            }
          },
          child: Container(
            padding: EdgeInsets.all(20.0),
            margin: EdgeInsets.all(5.0),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(15.0),
              color: Color(0xFF6174FF),
              boxShadow: [
                BoxShadow(
                  color: Color(0x2A6174FF),
                  spreadRadius: 0.0,
                  blurRadius: 10.0,
                  offset: Offset(0, 10),
                ),
              ],
            ),
            child: Container(
                width: 90.0,
                height: 20.0,
                alignment: Alignment.center,
                child: Icon(Icons.backspace_rounded,
                    size: 20.0, color: Colors.white)),
          ),
        ),
      ])
    ]);
  }
}
