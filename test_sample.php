<?php
class StudentController
{
    public function index()
    {
        $data['setting']       = json_encode($this->setting);
        $data['classlist']     = $this->class_model->get();
        $data['incidentlist']  = $this->studentbehaviour_model->get();
        $data['title']         = 'Student Behaviour';
        $data['subtitle']      = 'List';
        $this->load->view('header', $data);
        $this->load->view('student/index', $data);
    }

    public function add()
    {
        $name    = $this->input->post('name');
        $age     = $this->input->post('age');
        $email   = $this->input->post('email');
        $result  = $this->student_model->insert(['name' => $name, 'age' => $age, 'email' => $email]);
        echo json_encode(['status' => $result]);
    }
}
