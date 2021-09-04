import 'package:flutter/material.dart';

class SocketInputText extends StatefulWidget {
  final TextEditingController controller;
  final VoidCallback action;
  final bool state;
  bool changeState = false;

  SocketInputText({Key? key, required this.controller, required this.action, required this.state})
      : super(key: key);

  @override
  _SocketInputTextState createState() => _SocketInputTextState();
}

class _SocketInputTextState extends State<SocketInputText> {
  TextEditingController ipAddr = TextEditingController();
  TextEditingController port = TextEditingController();

  changed(_) {
    final ipPort = ipAddr.text + ':' + port.text;
    widget.controller.text = ipPort;

    setState(() {
        widget.changeState = true;
    });
  }

  @override
  void initState() {
    final initIpPort = widget.controller.text;
    final ipPort = initIpPort.split(':');
    if (ipPort.length == 2) {
      ipAddr.text = ipPort[0];
      port.text = ipPort[1];
    }

    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;

    return Container(
      height: 50.0,
      width: 300.0,
      padding: const EdgeInsets.symmetric(horizontal: 20.0),
      decoration: BoxDecoration(
        color: const Color(0x40ADB0BB),
        borderRadius: BorderRadius.circular(15.0)
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: <Widget>[
          Expanded(
            child: TextField(
              style: TextStyle(fontSize: 16.0),
              textAlign: TextAlign.center,
              controller: ipAddr,
              onChanged: changed,
              decoration: InputDecoration(
                hintText: '255.255.255.255',
                hintStyle: TextStyle(color: Color(0xFF1C1939).withOpacity(0.5)),
                border: InputBorder.none,
                filled: false,
                isDense: true,
              ),
            ),
          ),
          Text(':', style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16.0)),
          Container(
            width: 50,
            child: TextField(
              style: TextStyle(fontSize: 16.0),
              textAlign: TextAlign.center,
              controller: port,
              onChanged: changed,
              decoration: InputDecoration(
                hintText: '8888',
                hintStyle: TextStyle(color: Color(0xFF1C1939).withOpacity(0.5)),
                border: InputBorder.none,
                filled: false,
                isDense: true,
              ),
            ),
          ),
          if (widget.changeState)
          GestureDetector(
            onTap: widget.action,
            child: Container(
              width: 20.0,
              child: Icon(
                Icons.add_circle_outline,
                color: color.primary,
              )
            ),
          ),
          if (!widget.changeState && widget.state)
          GestureDetector(
            onTap: widget.action,
            child: Container(
              width: 20.0,
              child: Icon(
                Icons.link,
                color: Color(0xff0ee50a),
              )
            ),
          ),
          if (!widget.changeState && !widget.state)
          GestureDetector(
            onTap: widget.action,
            child: Container(
              width: 20.0,
              child: Icon(
                Icons.link_off,
                color: Colors.red,
              )
            ),
          ),
        ],
      ),
    );
  }
}
