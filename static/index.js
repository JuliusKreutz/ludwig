function remove(name) {
    let modal = document.getElementById("modal");
    let modalContent = document.getElementById("modal-content");
    modalContent.innerHTML = "";

    let text = document.createTextNode("You really wanna remove " + name);
    modalContent.appendChild(text);

    let yes = document.createElement("button");
    yes.innerHTML = "Yes";
    yes.onclick = async (_) => {
        await fetch("/remove", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                name: name,
                path: window.location.pathname,
            }),
        });

        location.reload();
    };
    modalContent.appendChild(yes);

    let cancel = document.createElement("button");
    cancel.innerHTML = "Cancel";
    cancel.onclick = (_) => {
        modal.style.display = "none";
    };
    modalContent.appendChild(cancel);

    modal.style.display = "block";
}

function newFolder() {
    let modal = document.getElementById("modal");
    let modalContent = document.getElementById("modal-content");
    modalContent.innerHTML = "";

    let text = document.createTextNode("Enter the name");
    modalContent.appendChild(text);

    let input = document.createElement("input");
    input.type = "text";
    modalContent.appendChild(input);

    let ok = document.createElement("button");
    ok.innerHTML = "Ok";
    ok.onclick = async (_) => {
        await fetch("/create", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                name: input.value,
                path: window.location.pathname,
            }),
        });

        location.reload();
    };
    modalContent.appendChild(ok);

    let cancel = document.createElement("button");
    cancel.innerHTML = "Cancel";
    cancel.onclick = (_) => {
        modal.style.display = "none";
    };
    modalContent.appendChild(cancel);

    modal.style.display = "block";
}

function uploadFiles() {
    let input = document.createElement("input");
    input.type = "file";
    input.multiple = true;

    input.onchange = async (_) => {
        let data = new FormData();

        data.append("path", window.location.pathname);

        for (var i = 0; i < input.files.length; i++) {
            data.append("file", input.files[i]);
        }

        await fetch("/upload", {
            method: "POST",
            body: data,
        });

        location.reload();
    };

    input.click();
    return false;
}
