
function getColor(n) {
  if (n == 0) return 'red';
  if (n <= 3) return 'yellow';
  if (n <= 6) return 'gray';
  if (n <= 10) return 'orange';
  return 'black';
}

class RunsPanel extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      devs: []
    };
  }

  updateComponent() {
    fetch("http://localhost:3030/runs")
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
                  top: 10 + this.state.devs.indexOf(dev) * 50,
                  transition: "1s ease top",
                }}
                >
                <div className="cell colocacao" 
                style={{
                  backgroundColor: getColor(dev.color)
                }}
                >{dev.placement}</div>
                <div className="cell nome_escola">{dev.escola}</div>
                <div className="cell nome_time">{dev.team_name}</div>
                <div className="cell problema">{dev.problem}</div>
                <div className="cell resposta">{dev.result}</div>
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
