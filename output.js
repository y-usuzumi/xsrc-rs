import axios from "axios";
class budgets {
    constructor(_super) {
        (this)._super = _super;
        (this)._url = (((this)._super).url) + ("/budgets");
    }
    async all() {
        return axios({
            "method": "get",
            "url": ((this)._super).url
        });
    }
}
class users {
    constructor(_super) {
        (this)._super = _super;
        (this)._url = (((this)._super).url) + ("/users/");
    }
    async all() {
        return axios({
            "method": "get",
            "url": ((this)._super).url
        });
    }
    async get(detail) {
        return axios({
            "method": "get",
            "url": ((((this)._super).url) + ("/")) + (id),
            "params": {
                "detail": detail
            }
        });
    }
    async create() {
        return axios({
            "method": "post",
            "url": ((this)._super).url,
            "data": {
                "username": username,
                "password": password
            }
        });
    }
    async update() {
        return axios({
            "method": "put",
            "url": (((((this)._super).url) + ("/")) + (id)) + ("/")
        });
    }
    get budgets() {
        return new (budgets)(this);
    }
}
class XSClient {
    constructor(url) {
        (this)._url = url;
    }
    get users() {
        return new (users)(this);
    }
}

