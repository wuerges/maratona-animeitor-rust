
function getColor(n) {
  if (n == 0) return "vermelho";
  if (n <= 3) return "ouro";
  if (n <= 6) return "prata";
  if (n <= 10) return "bronze";
  return "";
}

function getImgEscola(e) {
  return <img 
          className="logoimg" 
          src={"assets/logos/" + e.toLowerCase() + ".png"}
          alt={e.split('/').join(' /')} />
}

function getImgTeam(l) {
  return <p> {l}</p>
}

function getAnswer(t) {
  if (t == "Yes") {
    return <img src="assets/yes.png" />
  }
  else if (t == "No") {
    return <img src="assets/no.png" />
  }
  else if (t == "Wait") {
    return <img src="assets/question.png" />
  }
  return "Error";
}

class RunsPanel extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      devs: []
    };
  }

  updateComponent() {
    fetch("/runs")
    .then(res => res.json())
    .then(
      (result) => {
        console.log("Obtained results from api:", result)
        this.setState({devs: result});
      },
      // Nota: É importante lidar com os erros aqui
      // em vez de um bloco catch() para não recebermos
      // exceções de erros dos componentes.
      (error) => {
        console.log("ajax request error:", error);
        // this.setState({
        //   isLoaded: true,
        //   error
        // });
      }
    )
  }

  componentDidMount() {
    console.log("component did mount!");
    setInterval(() => {
      this.updateComponent();
    }, 1000);
  }

  render() {
    return (
      <>
        <div border="1">
          {this.state.devs.map((dev) => (
              <div className="run"
              key={dev.id}
              style={{
                  zIndex: dev.id,
                  position: "absolute",
                  top: 10 + this.state.devs.indexOf(dev) * 90,
                  transition: "1s ease top",
                }}
                >
                {/* <div className="cell logo">{getImgTeam(dev.team_login)}</div> */}
                <div className={"cell colocacao " + getColor(dev.placement)}>{dev.placement}</div>
                <div className="cell time">
                  <div className="nomeEscola">{dev.escola}</div>
                  <div className="nomeTime">{dev.team_name}</div>
                </div>
                <div className="cell problema">{dev.problem}</div>
                <div className="cell resposta">{getAnswer(dev.result)} </div>
            </div>
          ))}
        </div>
      </>
    );
  }
}
  
ReactDOM.render(
  <RunsPanel />,
  document.getElementById('root')
);
