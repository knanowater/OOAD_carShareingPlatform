function base64UrlDecode(base64Url) {
    const base64 = base64Url.replace(/-/g, "+").replace(/_/g, "/");
    const decodedData = atob(base64);
    try {
        return decodeURIComponent(
            decodedData
                .split("")
                .map((c) => {
                    return "%" + ("00" + c.charCodeAt(0).toString(16)).slice(-2);
                })
                .join("")
        );
    } catch (e) {
        console.error("디코딩 중 오류 발생:", e);
        return decodedData;
    }
}

async function checkAndAddDashboardButton() {
    const headerMenu = document.getElementById("header-menu");

    const response = await fetch("/api/isAdmin", {
        method: "GET",
        headers: {
            Authorization: `Bearer ${localStorage.getItem("token") === null ? "" : localStorage.getItem("token")}`,
            "Content-Type": "application/json",
        },
    });

    if (response.ok) {
        const isAdmin = await response.json();
        if (isAdmin) {
            const dashboardLink = document.createElement("a");
            dashboardLink.href = "/admin";
            dashboardLink.className = "text-gray-600 hover:text-blue-600";
            dashboardLink.textContent = "관리자 메뉴";

            headerMenu.appendChild(dashboardLink);
        }
    }
}

window.addEventListener("DOMContentLoaded", () => {
    // 관리자 대시보드 버튼 추가
    {
        checkAndAddDashboardButton();
    }
    // JWT 디코딩
    {
        const token = localStorage.getItem("token");
        const loginButtonContainer = document.querySelector(".flex.items-center.space-x-4");

        if (token) {
            try {
                const payloadBase64Url = token.split(".")[1];
                const payloadJson = base64UrlDecode(payloadBase64Url);
                const payload = JSON.parse(payloadJson);
                const username = payload.name;

                loginButtonContainer.innerHTML = `
        <span class="text-gray-600 font-medium">안녕하세요, ${username}님</span>
        <button class="px-4 py-2 rounded-full bg-red-600 text-white hover:bg-red-700 transition" onclick="logout()">로그아웃</button>
    `;
            } catch (error) {
                console.error("JWT 디코딩 실패:", error);
                alert("로그인 정보를 불러오는 데 실패했습니다.");
            }
        } else {
            loginButtonContainer.innerHTML = `
    <button class="px-4 py-2 rounded-full bg-blue-600 text-white hover:bg-blue-700 transition" onclick="location.href='/login'">로그인 / 회원가입</button>
    `;
        }
    }
});

// 로그아웃 함수
function logout() {
    localStorage.removeItem("token");
    location.reload();
}
