import { EditorView, basicSetup } from "codemirror";
import { json, jsonParseLinter } from "@codemirror/lang-json";
import { linter } from "@codemirror/lint";

var jsonEditorView = null;

window.initJsonEditor = (elementId, initialValue, onDocChange) => {
    const parentElement = document.getElementById(elementId);
    
    jsonEditorView = new EditorView({
        doc: initialValue,
        extensions: [
            basicSetup,
            json(), // Enables JSON syntax highlighting and smart indenting
            linter(jsonParseLinter()), // Enables real-time JSON validation errors
            EditorView.updateListener.of((update) => {
                if (update.docChanged) {
                    onDocChange(update.state.doc.toString());
                }
            })
        ],
        parent: parentElement
    });
};

window.setJsonEditorValue = (newValue) => {
    if (jsonEditorView) {
        jsonEditorView.dispatch({
            changes: {
                from: 0, 
                to: jsonEditorView.state.doc.length, 
                insert: newValue
            }
        });
    }
};

